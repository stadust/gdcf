use futures::{Async, Future};

use gdcf_model::{song::NewgroundsSong, user::Creator};

use crate::{
    api::{client::MakeRequest, ApiClient},
    cache::{Cache, CacheEntry, CanCache, Store},
    error::GdcfError,
    future::GdcfFuture,
    upgrade::{Upgrade, UpgradeMode},
    Gdcf,
};
use failure::_core::hint::unreachable_unchecked;

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
                    UpgradeMode::UpgradeCached(_) | UpgradeMode::UpgradeOutdated(..) => true,
                    UpgradeMode::UpgradeMissing(..) => false,
                },
            UpgradeFuture::Exhausted => false,
        }
    }

    fn into_cached(self) -> Result<Result<Self::Item, Self>, Self::Error>
    where
        Self: Sized,
    {
        if !self.has_result_cached() {
            return Ok(Err(self))
        }

        match self {
            UpgradeFuture::WaitingOnInner { gdcf, inner_future, .. } => {
                let base = match inner_future.into_cached()? {
                    Ok(base) => base,
                    _ => unreachable!(),
                };

                Ok(Ok(match base {
                    CacheEntry::Cached(to_upgrade, meta) =>
                        CacheEntry::Cached(
                            match temporary_upgrade(&gdcf.cache(), to_upgrade) {
                                Ok((upgraded, _)) => upgraded,
                                _ => unreachable!(),
                            },
                            meta,
                        ),
                    cache_entry => cache_entry.map_empty(),
                }))
            },
            UpgradeFuture::Extending(cache, meta, upgrade_mode) =>
                match upgrade_mode {
                    UpgradeMode::UpgradeCached(cached) => Ok(Ok(CacheEntry::Cached(cached, meta))),
                    UpgradeMode::UpgradeOutdated(to_upgrade, upgrade, _) => Ok(Ok(CacheEntry::Cached(to_upgrade.upgrade(upgrade).0, meta))),
                    UpgradeMode::UpgradeMissing(..) => unreachable!(),
                },
            UpgradeFuture::Exhausted => unreachable!(),
        }
    }

    fn peek_cached<F: FnOnce(Self::ToPeek) -> Self::ToPeek>(self, f: F) -> Self {
        if !self.has_result_cached() {
            return self
        }

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

#[allow(missing_debug_implementations)]
pub enum MultiUpgradeFuture<From, A, C, Into, U>
where
    A: ApiClient + MakeRequest<U::Request>,
    C: Cache + Store<Creator> + Store<NewgroundsSong> + CanCache<U::Request>,
    From: GdcfFuture<Item = CacheEntry<Vec<U>, C::CacheEntryMeta>, Error = GdcfError<A::Err, C::Err>>,
    U: Upgrade<C, Into>,
{
    WaitingOnInner {
        gdcf: Gdcf<A, C>,
        forced_refresh: bool,
        has_result_cached: bool,
        inner_future: From,
    },
    Extending(C, C::CacheEntryMeta, Vec<UpgradeMode<A, C, Into, U>>),
    Exhausted,
}

impl<From, A, C, Into, U> MultiUpgradeFuture<From, A, C, Into, U>
where
    A: ApiClient + MakeRequest<U::Request>,
    C: Cache + Store<Creator> + Store<NewgroundsSong> + CanCache<U::Request>,
    From: GdcfFuture<Item = CacheEntry<Vec<U>, C::CacheEntryMeta>, Error = GdcfError<A::Err, C::Err>, ToPeek = Vec<U>>,
    U: Upgrade<C, Into>,
{
    pub fn new(gdcf: Gdcf<A, C>, forced_refresh: bool, inner_future: From) -> Self {
        let mut has_result_cached = false;

        MultiUpgradeFuture::WaitingOnInner {
            inner_future: Self::peek_inner(&gdcf.cache(), inner_future, |cached| {
                has_result_cached = true;
                cached
            }),
            gdcf,
            has_result_cached,
            forced_refresh,
        }
    }

    fn peek_inner(cache: &C, inner: From, f: impl FnOnce(Vec<Into>) -> Vec<Into>) -> From {
        inner.peek_cached(move |e: Vec<U>| {
            match temporary_upgrade_all(cache, e) {
                Ok((upgraded, downgrades)) =>
                    f(upgraded)
                        .into_iter()
                        .zip(downgrades.into_iter())
                        .map(|(upgraded, downgrade)| U::downgrade(upgraded, downgrade).0)
                        .collect(),
                Err(failed) => failed,
            }
        })
    }
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
            MultiUpgradeFuture::WaitingOnInner {
                gdcf,
                forced_refresh,
                has_result_cached,
                mut inner_future,
            } => {
                match inner_future.poll()? {
                    Async::NotReady =>
                        (
                            Async::NotReady,
                            MultiUpgradeFuture::WaitingOnInner {
                                gdcf,
                                has_result_cached,
                                forced_refresh,
                                inner_future,
                            },
                        ),
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
                                    .map(|object| UpgradeMode::new(object, &gdcf, forced_refresh))
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

impl<From, A, C, Into, U> GdcfFuture for MultiUpgradeFuture<From, A, C, Into, U>
where
    A: ApiClient + MakeRequest<U::Request>,
    C: Cache + Store<Creator> + Store<NewgroundsSong> + CanCache<U::Request>,
    From: GdcfFuture<Item = CacheEntry<Vec<U>, C::CacheEntryMeta>, Error = GdcfError<A::Err, C::Err>, ToPeek = Vec<U>>,
    U: Upgrade<C, Into>,
{
    type ToPeek = Vec<Into>;

    fn has_result_cached(&self) -> bool {
        match self {
            MultiUpgradeFuture::WaitingOnInner { has_result_cached, .. } => *has_result_cached,
            MultiUpgradeFuture::Extending(_, _, upgrade_modes) =>
                upgrade_modes.iter().all(|mode| {
                    match mode {
                        UpgradeMode::UpgradeCached(_) | UpgradeMode::UpgradeOutdated(..) => true,
                        UpgradeMode::UpgradeMissing(..) => false,
                    }
                }),
            MultiUpgradeFuture::Exhausted => false,
        }
    }

    fn into_cached(self) -> Result<Result<Self::Item, Self>, Self::Error>
    where
        Self: Sized,
    {
        if !self.has_result_cached() {
            return Ok(Err(self))
        }

        match self {
            MultiUpgradeFuture::WaitingOnInner { gdcf, inner_future, .. } => {
                let base = match inner_future.into_cached()? {
                    Ok(base) => base,
                    _ => unreachable!(),
                };

                Ok(Ok(match base {
                    CacheEntry::Cached(to_upgrade, meta) =>
                        CacheEntry::Cached(
                            match temporary_upgrade_all(&gdcf.cache(), to_upgrade) {
                                Ok((upgraded, _)) => upgraded,
                                _ => unreachable!(),
                            },
                            meta,
                        ),
                    cache_entry => cache_entry.map_empty(),
                }))
            },
            MultiUpgradeFuture::Extending(cache, meta, upgrade_modes) => {
                let mut result = Vec::new();

                for upgrade_mode in upgrade_modes {
                    match upgrade_mode {
                        UpgradeMode::UpgradeCached(cached) => result.push(cached),
                        UpgradeMode::UpgradeOutdated(to_upgrade, upgrade, _) => result.push(to_upgrade.upgrade(upgrade).0),
                        UpgradeMode::UpgradeMissing(..) => unreachable!(),
                    }
                }

                Ok(Ok(CacheEntry::Cached(result, meta)))
            },
            MultiUpgradeFuture::Exhausted => unreachable!(),
        }
    }

    fn peek_cached<F: FnOnce(Self::ToPeek) -> Self::ToPeek>(self, f: F) -> Self {
        if !self.has_result_cached() {
            return self
        }

        match self {
            MultiUpgradeFuture::WaitingOnInner {
                gdcf,
                forced_refresh,
                has_result_cached,
                inner_future,
            } =>
                MultiUpgradeFuture::WaitingOnInner {
                    forced_refresh,
                    has_result_cached,
                    inner_future: Self::peek_inner(&gdcf.cache(), inner_future, f),
                    gdcf,
                },
            MultiUpgradeFuture::Extending(cache, meta, upgrade_modes) => {
                let mut upgraded = Vec::new();
                let mut downgrades = Vec::new();
                let mut futures = Vec::new();

                let mut failed = Vec::new();

                for upgrade_mode in upgrade_modes {
                    if !failed.is_empty() {
                        failed.push(upgrade_mode)
                    } else {
                        match upgrade_mode {
                            UpgradeMode::UpgradeCached(cached) => {
                                upgraded.push(cached);
                                downgrades.push(None);
                                futures.push(None);
                            },
                            UpgradeMode::UpgradeOutdated(to_upgrade, upgrade, future) => {
                                let (is_upgraded, downgrade) = to_upgrade.upgrade(upgrade);

                                upgraded.push(is_upgraded);
                                downgrades.push(Some(downgrade));
                                futures.push(Some(future));
                            },
                            UpgradeMode::UpgradeMissing(..) => {
                                while !upgraded.is_empty() {
                                    let upgraded = upgraded.remove(0);
                                    let downgrade = downgrades.remove(0);
                                    let future = futures.remove(0);

                                    failed.push(match downgrade {
                                        None => UpgradeMode::UpgradeCached(upgraded),
                                        Some(downgrade) => {
                                            let (downgraded, upgrade) = U::downgrade(upgraded, downgrade);

                                            UpgradeMode::UpgradeOutdated(downgraded, upgrade, future.unwrap())
                                        },
                                    });
                                }

                                failed.push(upgrade_mode)
                            },
                        }
                    }
                }

                let upgrade_modes = if failed.is_empty() {
                    upgraded = f(upgraded);
                    upgraded
                        .into_iter()
                        .zip(downgrades)
                        .zip(futures)
                        .map(|((upgraded, downgrade), future)| {
                            match downgrade {
                                None => UpgradeMode::UpgradeCached(upgraded),
                                Some(downgrade) => {
                                    let (downgraded, upgrade) = U::downgrade(upgraded, downgrade);

                                    UpgradeMode::UpgradeOutdated(downgraded, upgrade, future.unwrap())
                                },
                            }
                        })
                        .collect()
                } else {
                    failed
                };

                MultiUpgradeFuture::Extending(cache, meta, upgrade_modes)
            },
            MultiUpgradeFuture::Exhausted => MultiUpgradeFuture::Exhausted,
        }
    }
}

fn temporary_upgrade<C: Cache + CanCache<U::Request>, Into, U: Upgrade<C, Into>>(cache: &C, to_upgrade: U) -> Result<(Into, U::From), U> {
    let upgrade = match U::upgrade_request(to_upgrade.current()) {
        Some(request) =>
            match cache.lookup_request(&request) {
                Ok(CacheEntry::Cached(cached_result, _)) =>
                    match U::lookup_upgrade(to_upgrade.current(), cache, cached_result) {
                        Ok(upgrade) => upgrade,
                        _ => return Err(to_upgrade), // cache error on upgrade lookup
                    },
                Ok(CacheEntry::Missing) => return Err(to_upgrade), // no information about the upgrade was cached
                Ok(_) =>
                    match U::default_upgrade() {
                        // upgrade was marked/deduced as absent
                        Some(upgrade) => upgrade,
                        _ => return Err(to_upgrade),
                    },
                _ => return Err(to_upgrade), // error during request lookup
            },
        None =>
            match U::default_upgrade() {
                Some(upgrade) => upgrade,
                _ => return Err(to_upgrade), /* no upgrade request and no default (incorrect Upgrade impl, but that's something we check
                                              * later) */
            },
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
