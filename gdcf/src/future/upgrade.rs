use crate::{
    api::{client::MakeRequest, ApiClient},
    cache::{Cache, CacheEntry, CanCache, Store},
    error::GdcfError,
    future::GdcfFuture,
    upgrade::{Upgrade, UpgradeMode},
    Gdcf,
};
use futures::{Async, Future};
use gdcf_model::{song::NewgroundsSong, user::Creator};

#[allow(missing_debug_implementations)]
pub enum UpgradeFuture<From, A, C, Into, U>
where
    A: ApiClient + MakeRequest<U::Request>,
    C: Cache + Store<Creator> + Store<NewgroundsSong> + CanCache<U::Request>,
    From: GdcfFuture<Item = CacheEntry<U, C::CacheEntryMeta>, Error = GdcfError<A::Err, C::Err>>,
    U: Upgrade<C, Into>,
{
    WaitingOnInner(Gdcf<A, C>, bool, From),
    Extending(C, C::CacheEntryMeta, UpgradeMode<A, C, Into, U>),
    Exhausted,
}

#[allow(missing_debug_implementations)]
pub enum MultiUpgradeFuture<From, A, C, Into, U>
where
    A: ApiClient + MakeRequest<U::Request>,
    C: Cache + Store<Creator> + Store<NewgroundsSong> + CanCache<U::Request>,
    From: GdcfFuture<Item = CacheEntry<Vec<U>, C::CacheEntryMeta>, Error = GdcfError<A::Err, C::Err>>,
    U: Upgrade<C, Into>,
{
    WaitingOnInner(Gdcf<A, C>, bool, From),
    Extending(C, C::CacheEntryMeta, Vec<UpgradeMode<A, C, Into, U>>),
    Exhausted,
}

impl<From, A, C, Into, U> Future for MultiUpgradeFuture<From, A, C, Into, U>
where
    A: ApiClient + MakeRequest<U::Request>,
    C: Cache + Store<Creator> + Store<NewgroundsSong> + CanCache<U::Request>,
    From: GdcfFuture<Item = CacheEntry<Vec<U>, C::CacheEntryMeta>, Error = GdcfError<A::Err, C::Err>>,
    U: Upgrade<C, Into>,
{
    type Error = GdcfError<A::Err, C::Err>;
    type Item = CacheEntry<Vec<Into>, C::CacheEntryMeta>;

    fn poll(&mut self) -> Result<Async<Self::Item>, Self::Error> {
        let (return_value, next_state) = match self {
            MultiUpgradeFuture::WaitingOnInner(gdcf, forces_refresh, inner) =>
                match inner.poll()? {
                    Async::Ready(base) =>
                        match base {
                            CacheEntry::DeducedAbsent => (Async::Ready(CacheEntry::DeducedAbsent), MultiUpgradeFuture::Exhausted),
                            CacheEntry::MarkedAbsent(meta) => (Async::Ready(CacheEntry::MarkedAbsent(meta)), MultiUpgradeFuture::Exhausted),
                            CacheEntry::Missing => unreachable!(),
                            CacheEntry::Cached(objects, meta) => {
                                // TODO: figure out if this is really needed
                                futures::task::current().notify();

                                (
                                    Async::NotReady,
                                    MultiUpgradeFuture::Extending(
                                        gdcf.cache(),
                                        meta,
                                        objects
                                            .into_iter()
                                            .map(|object| UpgradeMode::new(object, gdcf, *forces_refresh))
                                            .collect::<Result<Vec<_>, _>>()?,
                                    ),
                                )
                            },
                        },
                    Async::NotReady => return Ok(Async::NotReady),
                },
            MultiUpgradeFuture::Extending(ref cache, _, ref mut upgrades) => {
                let mut are_we_there_yet = true;

                for extension in upgrades {
                    match extension {
                        UpgradeMode::UpgradeOutdated(_, _, ref mut future) | UpgradeMode::UpgradeMissing(_, ref mut future) =>
                            match future.poll()? {
                                Async::NotReady => are_we_there_yet = false,
                                Async::Ready(CacheEntry::Missing) => unreachable!(),
                                Async::Ready(CacheEntry::DeducedAbsent) | Async::Ready(CacheEntry::MarkedAbsent(_)) =>
                                    if let UpgradeMode::UpgradeMissing(to_extend, _) | UpgradeMode::UpgradeOutdated(to_extend, ..) =
                                        std::mem::replace(extension, UpgradeMode::FixMeItIsLateAndICannotThinkOfABetterSolution)
                                    {
                                        std::mem::replace(
                                            extension,
                                            UpgradeMode::UpgradeCached(
                                                to_extend.upgrade(U::default_upgrade().ok_or(GdcfError::ConsistencyAssumptionViolated)?),
                                            ),
                                        );
                                    },
                                Async::Ready(CacheEntry::Cached(request_result, _)) => {
                                    if let UpgradeMode::UpgradeMissing(to_extend, _) | UpgradeMode::UpgradeOutdated(to_extend, ..) =
                                        std::mem::replace(extension, UpgradeMode::FixMeItIsLateAndICannotThinkOfABetterSolution)
                                    {
                                        let upgrade =
                                            U::lookup_upgrade(to_extend.current(), cache, request_result).map_err(GdcfError::Cache)?;

                                        std::mem::replace(extension, UpgradeMode::UpgradeCached(to_extend.upgrade(upgrade)));
                                    }
                                },
                            },
                        _ => (),
                    }
                }

                if are_we_there_yet {
                    if let MultiUpgradeFuture::Extending(_, meta, extensions) = std::mem::replace(self, MultiUpgradeFuture::Exhausted) {
                        return Ok(Async::Ready(CacheEntry::Cached(
                            extensions
                                .into_iter()
                                .map(|extension| {
                                    if let UpgradeMode::UpgradeCached(object) = extension {
                                        object
                                    } else {
                                        unreachable!()
                                    }
                                })
                                .collect(),
                            meta,
                        )))
                    } else {
                        unreachable!()
                    }
                } else {
                    return Ok(Async::NotReady)
                }
            },
            _ => unimplemented!(),
            MultiUpgradeFuture::Exhausted => panic!("Future already polled to completion"),
        };

        std::mem::replace(self, next_state);

        Ok(return_value)
    }
}

// TODO: this impl is tricky
impl<From, A, C, Into, U> GdcfFuture for MultiUpgradeFuture<From, A, C, Into, U>
where
    A: ApiClient + MakeRequest<U::Request>,
    C: Cache + Store<Creator> + Store<NewgroundsSong> + CanCache<U::Request>,
    From: GdcfFuture<Item = CacheEntry<Vec<U>, C::CacheEntryMeta>, Error = GdcfError<A::Err, C::Err>>,
    U: Upgrade<C, Into>,
{
    type Extension = ();

    fn cached_extension(&self) -> Option<&Self::Extension> {
        unimplemented!();
    }

    fn has_result_cached(&self) -> bool {
        unimplemented!()
    }

    fn into_cached(self) -> Option<Self::Item> {
        unimplemented!()
    }
}

impl<From, A, C, Into, U> Future for UpgradeFuture<From, A, C, Into, U>
where
    A: ApiClient + MakeRequest<U::Request>,
    C: Cache + Store<Creator> + Store<NewgroundsSong> + CanCache<U::Request>,
    From: GdcfFuture<Item = CacheEntry<U, C::CacheEntryMeta>, Error = GdcfError<A::Err, C::Err>>,
    U: Upgrade<C, Into>,
{
    type Error = GdcfError<A::Err, C::Err>;
    type Item = CacheEntry<Into, C::CacheEntryMeta>;

    fn poll(&mut self) -> Result<Async<Self::Item>, Self::Error> {
        let (return_value, next_state) = match self {
            UpgradeFuture::WaitingOnInner(gdcf, forces_refresh, inner) =>
                match inner.poll()? {
                    Async::Ready(base) =>
                        match base {
                            CacheEntry::DeducedAbsent => (Async::Ready(CacheEntry::DeducedAbsent), UpgradeFuture::Exhausted),
                            CacheEntry::MarkedAbsent(meta) => (Async::Ready(CacheEntry::MarkedAbsent(meta)), UpgradeFuture::Exhausted),
                            CacheEntry::Missing => unreachable!(),
                            CacheEntry::Cached(object, meta) => {
                                // TODO: figure out if this is really needed
                                futures::task::current().notify();

                                (
                                    Async::NotReady,
                                    UpgradeFuture::Extending(gdcf.cache(), meta, UpgradeMode::new(object, gdcf, *forces_refresh)?),
                                )
                            },
                        },
                    Async::NotReady => return Ok(Async::NotReady),
                },
            UpgradeFuture::Extending(_, _, UpgradeMode::UpgradeCached(_)) => {
                if let UpgradeFuture::Extending(_, meta, UpgradeMode::UpgradeCached(object)) =
                    std::mem::replace(self, UpgradeFuture::Exhausted)
                {
                    return Ok(Async::Ready(CacheEntry::Cached(object, meta)))
                } else {
                    unreachable!()
                }
            },
            UpgradeFuture::Extending(_, _, UpgradeMode::UpgradeMissing(_, ref mut refresh_future))
            | UpgradeFuture::Extending(_, _, UpgradeMode::UpgradeOutdated(_, _, ref mut refresh_future)) =>
                match refresh_future.poll()? {
                    Async::NotReady => return Ok(Async::NotReady),
                    Async::Ready(CacheEntry::Missing) => unreachable!(),
                    Async::Ready(cache_entry) => {
                        let (cache, to_upgrade, meta) = match std::mem::replace(self, UpgradeFuture::Exhausted) {
                            UpgradeFuture::Extending(cache, meta, UpgradeMode::UpgradeMissing(base, _))
                            | UpgradeFuture::Extending(cache, meta, UpgradeMode::UpgradeOutdated(base, ..)) => (cache, base, meta),
                            _ => unreachable!(),
                        };

                        match cache_entry {
                            CacheEntry::DeducedAbsent | CacheEntry::MarkedAbsent(_) =>
                                return Ok(Async::Ready(CacheEntry::Cached(
                                    to_upgrade.upgrade(U::default_upgrade().ok_or(GdcfError::ConsistencyAssumptionViolated)?),
                                    meta,
                                ))),
                            CacheEntry::Cached(request_result, _) => {
                                let upgrade = U::lookup_upgrade(to_upgrade.current(), &cache, request_result).map_err(GdcfError::Cache)?;

                                return Ok(Async::Ready(CacheEntry::Cached(to_upgrade.upgrade(upgrade), meta)))
                            },
                            _ => unreachable!(),
                        }
                    },
                },
            _ => unreachable!(),
            UpgradeFuture::Exhausted => panic!("Future already polled to completion"),
        };

        std::mem::replace(self, next_state);

        Ok(return_value)
    }
}

// TODO: this impl is gonna be tricky as well
impl<From, A, C, Into, U> GdcfFuture for UpgradeFuture<From, A, C, Into, U>
where
    A: ApiClient + MakeRequest<U::Request>,
    C: Cache + Store<Creator> + Store<NewgroundsSong> + CanCache<U::Request>,
    From: GdcfFuture<Item = CacheEntry<U, C::CacheEntryMeta>, Error = GdcfError<A::Err, C::Err>>,
    U: Upgrade<C, Into>,
{
    type Extension = U::Upgrade;

    fn cached_extension(&self) -> Option<&Self::Extension> {
        match self {
            UpgradeFuture::WaitingOnInner(_, _, inner_future) => {
                let inner = inner_future.cached_extension()?;


                ()
            },
            UpgradeFuture::Extending(..) => {},
            UpgradeFuture::Exhausted => (),
        }

        unimplemented!()
    }

    fn has_result_cached(&self) -> bool {
        unimplemented!()
    }

    fn into_cached(self) -> Option<Self::Item> {
        unimplemented!()
        /*match self {
            UpgradeFuture::WaitingOnInner(gdcf, _, inner) =>
                if let Some(CacheEntry::Cached(to_extend, meta)) = inner.into_cached() {
                    let to_extend: U = to_extend;
                    let req = to_extend.extension_request();
                    let cache = gdcf.cache();

                    cache
                        .lookup_request(&req)
                        .ok()
                        .and_then(|result| result.into())
                        .and_then(|result| to_extend.lookup_extension(&cache, result).ok())
                        .map(|extension| CacheEntry::Cached(to_extend.extend(extension), meta))
                } else {
                    None
                },
            UpgradeFuture::Extending(_, meta, ext_mode) =>
                match ext_mode {
                    UpgradeMode::ExtensionWasCached(extended) => Some(CacheEntry::Cached(extended, meta)),
                    UpgradeMode::ExtensionWasOutdated(to_extend, extension, _) =>
                        Some(CacheEntry::Cached(to_extend.extend(extension), meta)),
                    _ => None,
                },
            UpgradeFuture::Exhausted => None,
        }*/
    }
}
