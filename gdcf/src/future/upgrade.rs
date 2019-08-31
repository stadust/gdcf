use futures::{Async, Future};

use gdcf_model::{song::NewgroundsSong, user::Creator};

use crate::{
    api::{client::MakeRequest, ApiClient},
    cache::{Cache, CacheEntry, CanCache, Store},
    error::GdcfError,
    future::GdcfFuture,
    upgrade::{
        Upgrade,
        UpgradeMode::{self, UpgradeCached, UpgradeMissing},
    },
    Gdcf,
};

#[allow(missing_debug_implementations)]
pub enum UpgradeFuture<From, A, C, Into, U>
where
    A: ApiClient + MakeRequest<U::Request>,
    C: Cache + Store<Creator> + Store<NewgroundsSong> + CanCache<U::Request>,
    From: GdcfFuture<Item = CacheEntry<U, C::CacheEntryMeta>, Error = GdcfError<A::Err, C::Err>>,
    U: Upgrade<C, Into>,
{
    WaitingOnInner {
        gdcf: Gdcf<A, C>,
        forced_refresh: bool,
        has_result_cached: bool,
        inner_future: From,
    },
    Extending(C, C::CacheEntryMeta, UpgradeMode<A, C, Into, U>),
    Exhausted,
}

impl<From, A, C, Into, U> UpgradeFuture<From, A, C, Into, U>
where
    A: ApiClient + MakeRequest<U::Request>,
    C: Cache + Store<Creator> + Store<NewgroundsSong> + CanCache<U::Request>,
    From: GdcfFuture<Item = CacheEntry<U, C::CacheEntryMeta>, Error = GdcfError<A::Err, C::Err>, ToPeek = U>,
    U: Upgrade<C, Into>,
{
    pub fn new(gdcf: Gdcf<A, C>, forced_refresh: bool, inner_future: From) -> Self {
        let mut has_result_cached = false;

        UpgradeFuture::WaitingOnInner {
            forced_refresh,
            inner_future: Self::peek_inner(&gdcf.cache(), inner_future, |cached| {
                has_result_cached = true;
                cached
            }),
            gdcf,
            has_result_cached,
        }
    }

    fn peek_inner(cache: &C, inner: From, f: impl FnOnce(Into) -> Into) -> From {
        inner.peek_cached(move |peeked| {
            match temporary_upgrade(cache, peeked) {
                Ok((upgraded, downgrade)) => U::downgrade(f(upgraded), downgrade).0,
                Err(not_upgraded) => not_upgraded,
            }
        })
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
        let (ready, new_self) = match std::mem::replace(self, UpgradeFuture::Exhausted) {
            UpgradeFuture::WaitingOnInner {
                gdcf,
                forced_refresh,
                has_result_cached,
                mut inner_future,
            } =>
                match inner_future.poll()? {
                    Async::NotReady =>
                        (
                            Async::NotReady,
                            UpgradeFuture::WaitingOnInner {
                                gdcf,
                                forced_refresh,
                                has_result_cached,
                                inner_future,
                            },
                        ),
                    Async::Ready(CacheEntry::Cached(inner_object, meta)) => {
                        // TODO: figure out if this is really needed
                        futures::task::current().notify();
                        (
                            Async::NotReady,
                            UpgradeFuture::Extending(gdcf.cache(), meta, UpgradeMode::new(inner_object, &gdcf, forced_refresh)?),
                        )
                    },
                    Async::Ready(cache_entry) => (Async::Ready(cache_entry.map_empty()), UpgradeFuture::Exhausted),
                },

            UpgradeFuture::Extending(_, meta, UpgradeMode::UpgradeCached(object)) =>
                (Async::Ready(CacheEntry::Cached(object, meta)), UpgradeFuture::Exhausted),

            UpgradeFuture::Extending(cache, meta, mut upgrade_mode) =>
                match upgrade_mode.future().unwrap().poll()? {
                    Async::NotReady => (Async::NotReady, UpgradeFuture::Extending(cache, meta, upgrade_mode)),
                    Async::Ready(cache_entry) =>
                        match upgrade_mode {
                            UpgradeMode::UpgradeMissing(to_upgrade, _) | UpgradeMode::UpgradeOutdated(to_upgrade, ..) => {
                                let upgrade = match cache_entry {
                                    CacheEntry::DeducedAbsent | CacheEntry::MarkedAbsent(_) =>
                                        U::default_upgrade().ok_or(GdcfError::ConsistencyAssumptionViolated)?,
                                    CacheEntry::Cached(request_result, _) =>
                                        U::lookup_upgrade(to_upgrade.current(), &cache, request_result).map_err(GdcfError::Cache)?,
                                    _ => unreachable!(),
                                };
                                let (upgraded, _) = to_upgrade.upgrade(upgrade);

                                (Async::Ready(CacheEntry::Cached(upgraded, meta)), UpgradeFuture::Exhausted)
                            },
                            _ => unreachable!(),
                        },
                },

            UpgradeFuture::Exhausted => panic!("Future already polled to completion"),
        };

        *self = new_self;

        Ok(ready)
    }
}

impl<From, A, C, Into, U> GdcfFuture for UpgradeFuture<From, A, C, Into, U>
where
    A: ApiClient + MakeRequest<U::Request>,
    C: Cache + Store<Creator> + Store<NewgroundsSong> + CanCache<U::Request>,
    From: GdcfFuture<Item = CacheEntry<U, C::CacheEntryMeta>, Error = GdcfError<A::Err, C::Err>, ToPeek = U>,
    U: Upgrade<C, Into>,
{
    type ToPeek = Into;

    fn has_result_cached(&self) -> bool {
        match self {
            UpgradeFuture::WaitingOnInner { has_result_cached, .. } => *has_result_cached,
            UpgradeFuture::Extending(_, _, upgrade_mode) =>
                match upgrade_mode {
                    UpgradeCached(_) | UpgradeMode::UpgradeOutdated(..) => true,
                    UpgradeMissing(..) => false,
                },
            UpgradeFuture::Exhausted => false,
        }
    }

    fn peek_cached<F: FnOnce(Self::ToPeek) -> Self::ToPeek>(self, f: F) -> Self {
        match self {
            UpgradeFuture::WaitingOnInner {
                gdcf,
                forced_refresh,
                has_result_cached,
                inner_future,
            } =>
                UpgradeFuture::WaitingOnInner {
                    forced_refresh,
                    has_result_cached,
                    inner_future: Self::peek_inner(&gdcf.cache(), inner_future, f),
                    gdcf,
                },
            UpgradeFuture::Extending(cache, meta, upgrade_mode) =>
                match upgrade_mode {
                    UpgradeMode::UpgradeCached(cached) => UpgradeFuture::Extending(cache, meta, UpgradeMode::UpgradeCached(f(cached))),
                    UpgradeMode::UpgradeOutdated(to_upgrade, upgrade, future) => {
                        let (upgraded, downgrade) = to_upgrade.upgrade(upgrade);
                        let (downgraded, upgrade) = U::downgrade(f(upgraded), downgrade);

                        UpgradeFuture::Extending(cache, meta, UpgradeMode::UpgradeOutdated(downgraded, upgrade, future))
                    },
                    UpgradeMode::UpgradeMissing(to_upgrade, future) =>
                        UpgradeFuture::Extending(cache, meta, UpgradeMode::UpgradeMissing(to_upgrade, future)),
                },
            UpgradeFuture::Exhausted => UpgradeFuture::Exhausted,
        }
    }
}

/*
// TODO: this impl is gonna be tricky as well
impl<From, A, C, Into, U> GdcfFuture for UpgradeFuture<From, A, C, Into, U>
where
    A: ApiClient + MakeRequest<U::Request>,
    C: Cache + Store<Creator> + Store<NewgroundsSong> + CanCache<U::Request>,
    From: GdcfFuture<Item = CacheEntry<U, C::CacheEntryMeta>, Error = GdcfError<A::Err, C::Err>, Upgrade=U::From>,
    U: Upgrade<C, Into>,
{
    type Upgrade = U::Upgrade;

    fn cached_extension(&self) -> Option<&Self::Upgrade> {
        match self {
            UpgradeFuture::WaitingOnInner(gdcf, _, inner_future) => {
                let inner = inner_future.cached_extension()?;
                let request = U::upgrade_request(inner)?;//.or_else(||U::default_upgrade())?;
                let cached_result: CacheEntry<_, _> = gdcf.cache().lookup_request(&request).ok()?; // FIXME: proper error handling
                let cached_result: Option<_> = cached_result.into();
                let lookup_result = U::lookup_upgrade(inner, &gdcf.cache(), cached_result?).ok()?;

                return Some(&lookup_result)
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
*/

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
        let (ready, new_self) = match std::mem::replace(self, MultiUpgradeFuture::Exhausted) {
            MultiUpgradeFuture::WaitingOnInner(gdcf, forces_refresh, mut inner) => {
                match inner.poll()? {
                    Async::NotReady => (Async::NotReady, MultiUpgradeFuture::WaitingOnInner(gdcf, forces_refresh, inner)),
                    Async::Ready(CacheEntry::Cached(cached_objects, meta)) => {
                        // TODO: figure out if this is really needed
                        futures::task::current().notify();

                        (
                            Async::NotReady,
                            MultiUpgradeFuture::Extending(
                                gdcf.cache(),
                                meta,
                                cached_objects
                                    .into_iter()
                                    .map(|object| UpgradeMode::new(object, &gdcf, forces_refresh))
                                    .collect::<Result<Vec<_>, _>>()?,
                            ),
                        )
                    },
                    Async::Ready(cache_entry) => (Async::Ready(cache_entry.map_empty()), MultiUpgradeFuture::Exhausted),
                }
            },

            MultiUpgradeFuture::Extending(cache, meta, mut entry_upgrade_modes) => {
                let mut done = Vec::new();
                let mut not_done = Vec::new();

                for mut upgrade_mode in entry_upgrade_modes {
                    match upgrade_mode {
                        UpgradeMode::UpgradeCached(cached) => done.push(cached),
                        mut upgrade_mode =>
                            match upgrade_mode.future().unwrap().poll()? {
                                Async::NotReady => not_done.push(upgrade_mode),
                                Async::Ready(cache_entry) => {
                                    let to_upgrade = upgrade_mode.to_upgrade().unwrap();
                                    let upgrade = match cache_entry {
                                        CacheEntry::MarkedAbsent(_) | CacheEntry::DeducedAbsent =>
                                            U::default_upgrade().ok_or(GdcfError::ConsistencyAssumptionViolated)?,
                                        CacheEntry::Cached(request_result, _) =>
                                            U::lookup_upgrade(to_upgrade.current(), &cache, request_result).map_err(GdcfError::Cache)?,
                                        _ => unreachable!(),
                                    };
                                    let (upgraded, _) = to_upgrade.upgrade(upgrade);

                                    done.push(upgraded);
                                },
                            },
                    }
                }

                if not_done.is_empty() {
                    (Async::Ready(CacheEntry::Cached(done, meta)), MultiUpgradeFuture::Exhausted)
                } else {
                    not_done.extend(done.into_iter().map(UpgradeMode::UpgradeCached));
                    (Async::NotReady, MultiUpgradeFuture::Extending(cache, meta, not_done))
                }
            },

            MultiUpgradeFuture::Exhausted => panic!("Future already polled to completion"),
        };

        *self = new_self;

        Ok(ready)
    }
}

// TODO: this impl is tricky
impl<From, A, C, Into, U> GdcfFuture for MultiUpgradeFuture<From, A, C, Into, U>
where
    A: ApiClient + MakeRequest<U::Request>,
    C: Cache + Store<Creator> + Store<NewgroundsSong> + CanCache<U::Request>,
    From: GdcfFuture<Item = CacheEntry<Vec<U>, C::CacheEntryMeta>, Error = GdcfError<A::Err, C::Err>, ToPeek = Vec<U>>,
    U: Upgrade<C, Into>,
{
    type ToPeek = Vec<Into>;

    fn has_result_cached(&self) -> bool {
        unimplemented!()
    }

    /*fn into_cached(self) -> Option<Self::Item> {
        unimplemented!()
    }*/

    fn peek_cached<F: FnOnce(Self::ToPeek) -> Self::ToPeek>(self, f: F) -> Self {
        match self {
            MultiUpgradeFuture::WaitingOnInner(gdcf, force_refresh, inner) => {
                let cache = gdcf.cache();

                let post_peek = inner.peek_cached(move |e: Vec<U>| {
                    match temporary_upgrade_all(&cache, e) {
                        Ok((upgraded, downgrades)) => {
                            f(upgraded)
                                .into_iter()
                                .zip(downgrades.into_iter())
                                .map(|(upgraded, downgrade)| U::downgrade(upgraded, downgrade).0)
                                .collect()
                        },
                        Err(failed) => failed
                    }
                });

                MultiUpgradeFuture::WaitingOnInner(gdcf, force_refresh, post_peek)
            },
            MultiUpgradeFuture::Extending(..) => unimplemented!(),
            MultiUpgradeFuture::Exhausted => MultiUpgradeFuture::Exhausted,
        }
    }
}

fn temporary_upgrade<C: Cache + CanCache<U::Request>, Into, U: Upgrade<C, Into>>(cache: &C, to_upgrade: U) -> Result<(Into, U::From), U> {
    let request = match U::upgrade_request(to_upgrade.current()) {
        Some(request) => request,
        _ => return Err(to_upgrade),
    };
    let cached_result = match cache.lookup_request(&request) {
        Ok(result) =>
            match result.into() {
                Some(result) => result,
                None => return Err(to_upgrade),
            },
        _ => return Err(to_upgrade),
    };

    let upgrade = match U::lookup_upgrade(to_upgrade.current(), &cache, cached_result) {
        Ok(upgrade) => upgrade,
        _ => return Err(to_upgrade),
    };

    Ok(to_upgrade.upgrade(upgrade))
}

fn temporary_upgrade_all<C: Cache + CanCache<U::Request>, Into, U: Upgrade<C, Into>>(
    cache: &C,
    to_upgrade: Vec<U>,
) -> Result<(Vec<Into>, Vec<U::From>), Vec<U>> {
    let mut temporarily_upgraded = Vec::new();
    let mut downgrades = Vec::new();

    let mut failed = Vec::new();

    for to_upgrade in to_upgrade {
        if !failed.is_empty() {
            failed.push(to_upgrade)
        } else {
            match temporary_upgrade(cache, to_upgrade) {
                Ok((upgraded, downgrade)) => {
                    temporarily_upgraded.push(upgraded);
                    downgrades.push(downgrade);
                },
                Err(not_upgraded) => {
                    // At this point, `failed` is still an empty vec!
                    while !temporarily_upgraded.is_empty() {
                        let upgraded = temporarily_upgraded.remove(0);
                        let downgrade = downgrades.remove(0);

                        failed.push(U::downgrade(upgraded, downgrade).0)
                    }

                    failed.push(not_upgraded)
                },
            }
        }
    }

    if !failed.is_empty() {
        Ok((temporarily_upgraded, downgrades))
    } else {
        Err(failed)
    }
}
