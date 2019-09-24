use futures::{Async, Future};
use log::{debug, error, trace, warn};

use crate::{
    api::{client::MakeRequest, request::Request, ApiClient},
    cache::{Cache, CacheEntry, CanCache},
    error::Error,
    future::{CloneCached, GdcfFuture},
    upgrade::{PendingUpgrade, Upgradable},
    Gdcf,
};

#[allow(missing_debug_implementations)]
pub struct UpgradeFuture<From, Into, U>
where
    From::ApiClient: MakeRequest<U::Request>,
    From::Cache: CanCache<U::Request>,
    From: GdcfFuture<GdcfItem = U>,
    U: Upgradable<From::Cache, Into>,
{
    gdcf: Gdcf<From::ApiClient, From::Cache>,
    forced_refresh: bool,
    state: UpgradeFutureState<From, Into, U>,
}

impl<From, Into, U> std::fmt::Debug for UpgradeFuture<From, Into, U>
where
    From::ApiClient: MakeRequest<U::Request>,
    From::Cache: CanCache<U::Request>,
    From: GdcfFuture<GdcfItem = U> + std::fmt::Debug,
    U: Upgradable<From::Cache, Into> + std::fmt::Debug,
    U::Upgrade: std::fmt::Debug,
    Into: std::fmt::Debug,
{
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        f.debug_struct("UpgradeFuture")
            .field("forced_refresh", &self.forced_refresh)
            .field("state", &self.state)
            .finish()
    }
}

impl<From, Into, U> UpgradeFuture<From, Into, U>
where
    From::ApiClient: MakeRequest<U::Request>,
    From::Cache: CanCache<U::Request>,
    From: GdcfFuture<GdcfItem = U>,
    U: Upgradable<From::Cache, Into>,
{
    pub fn upgrade_from(from: From) -> Self {
        let gdcf = from.gdcf();

        UpgradeFuture {
            forced_refresh: from.forcing_refreshes(),
            state: UpgradeFutureState::new(&gdcf.cache(), from),
            gdcf,
        }
    }

    pub fn upgrade<Into2>(self) -> UpgradeFuture<Self, Into2, Into>
    where
        Into: Upgradable<From::Cache, Into2>,
        From::ApiClient: MakeRequest<Into::Request>,
        From::Cache: CanCache<Into::Request>,
    {
        UpgradeFuture::upgrade_from(self)
    }
}

enum UpgradeFutureState<From, Into, U>
where
    From::ApiClient: MakeRequest<U::Request>,
    From::Cache: CanCache<U::Request>,
    From: GdcfFuture<GdcfItem = U>,
    U: Upgradable<From::Cache, Into>,
{
    WaitingOnInner {
        has_result_cached: bool,
        inner_future: From,
    },
    Extending(
        <From::Cache as Cache>::CacheEntryMeta,
        PendingUpgrade<From::ApiClient, From::Cache, Into, U>,
    ),
    Exhausted,
}

impl<From, Into, U> std::fmt::Debug for UpgradeFutureState<From, Into, U>
where
    From::ApiClient: MakeRequest<U::Request>,
    From::Cache: CanCache<U::Request>,
    From: GdcfFuture<GdcfItem = U> + std::fmt::Debug,
    U: Upgradable<From::Cache, Into> + std::fmt::Debug,
    U::Upgrade: std::fmt::Debug,
    Into: std::fmt::Debug,
{
    fn fmt(&self, fmt: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            UpgradeFutureState::WaitingOnInner {
                has_result_cached,
                inner_future,
            } =>
                fmt.debug_struct("WaitingOnInner")
                    .field("has_result_cached", has_result_cached)
                    .field("inner_future", inner_future)
                    .finish(),
            UpgradeFutureState::Extending(meta, upgrade_mode) => fmt.debug_tuple("Extending").field(meta).field(upgrade_mode).finish(),
            UpgradeFutureState::Exhausted => fmt.debug_tuple("Exhausted").finish(),
        }
    }
}

impl<From, Into, U> UpgradeFutureState<From, Into, U>
where
    From::ApiClient: MakeRequest<U::Request>,
    From::Cache: CanCache<U::Request>,
    From: GdcfFuture<GdcfItem = U>,
    U: Upgradable<From::Cache, Into>,
{
    pub fn new(cache: &From::Cache, inner_future: From) -> Self {
        let mut has_result_cached = false;

        UpgradeFutureState::WaitingOnInner {
            inner_future: Self::peek_inner(&cache, inner_future, |cached| {
                has_result_cached = true;
                cached
            }),
            has_result_cached,
        }
    }

    fn peek_inner(cache: &From::Cache, inner: From, f: impl FnOnce(Into) -> Into) -> From {
        inner.peek_cached(move |peeked| {
            match temporary_upgrade(cache, peeked) {
                Ok((upgraded, downgrade)) => U::downgrade(f(upgraded), downgrade).0,
                Err(not_upgraded) => not_upgraded,
            }
        })
    }
}

impl<From, Into, U> GdcfFuture for UpgradeFuture<From, Into, U>
where
    From::ApiClient: MakeRequest<U::Request>,
    From::Cache: CanCache<U::Request>,
    From: GdcfFuture<GdcfItem = U>,
    U: Upgradable<From::Cache, Into>,
{
    type ApiClient = From::ApiClient;
    type BaseRequest = <From as GdcfFuture>::BaseRequest;
    type Cache = From::Cache;
    type GdcfItem = Into;

    fn has_result_cached(&self) -> bool {
        match self.state {
            UpgradeFutureState::WaitingOnInner { has_result_cached, .. } => has_result_cached,
            UpgradeFutureState::Extending(_, ref upgrade_mode) =>
                match upgrade_mode {
                    PendingUpgrade::Cached(_) | PendingUpgrade::Outdated(..) => true,
                    PendingUpgrade::Missing(..) => false,
                },
            UpgradeFutureState::Exhausted => false,
        }
    }

    fn into_cached(
        self,
    ) -> Result<
        Result<CacheEntry<Self::GdcfItem, <Self::Cache as Cache>::CacheEntryMeta>, Self>,
        Error<<Self::ApiClient as ApiClient>::Err, <Self::Cache as Cache>::Err>,
    >
    where
        Self: Sized,
    {
        if !self.has_result_cached() {
            return Ok(Err(self))
        }

        match self.state {
            UpgradeFutureState::WaitingOnInner { inner_future, .. } => {
                let cache = self.gdcf.cache();
                let base = match inner_future.into_cached()? {
                    Ok(base) =>
                        base.map(|to_upgrade| {
                            match temporary_upgrade(&cache, to_upgrade) {
                                Ok((upgraded, _)) => upgraded,
                                _ => unreachable!(),
                            }
                        }),
                    _ => unreachable!(),
                };

                Ok(Ok(base))
            },
            UpgradeFutureState::Extending(meta, upgrade_mode) =>
                match upgrade_mode {
                    PendingUpgrade::Cached(cached) => Ok(Ok(CacheEntry::Cached(cached, meta))),
                    PendingUpgrade::Outdated(to_upgrade, upgrade, _) => Ok(Ok(CacheEntry::Cached(to_upgrade.upgrade(upgrade).0, meta))),
                    PendingUpgrade::Missing(..) => unreachable!(),
                },
            UpgradeFutureState::Exhausted => unreachable!(),
        }
    }

    fn new(gdcf: Gdcf<Self::ApiClient, Self::Cache>, request: &Self::BaseRequest) -> Result<Self, <Self::Cache as Cache>::Err> {
        Ok(UpgradeFuture {
            state: UpgradeFutureState::new(&gdcf.cache(), From::new(gdcf.clone(), request)?),
            gdcf,
            forced_refresh: request.forces_refresh(),
        })
    }

    fn gdcf(&self) -> Gdcf<Self::ApiClient, Self::Cache> {
        self.gdcf.clone()
    }

    fn forcing_refreshes(&self) -> bool {
        self.forced_refresh
    }

    fn poll(
        &mut self,
    ) -> Result<
        Async<CacheEntry<Self::GdcfItem, <Self::Cache as Cache>::CacheEntryMeta>>,
        Error<<Self::ApiClient as ApiClient>::Err, <Self::Cache as Cache>::Err>,
    > {
        let (ready, new_state) = match std::mem::replace(&mut self.state, UpgradeFutureState::Exhausted) {
            UpgradeFutureState::WaitingOnInner {
                has_result_cached,
                mut inner_future,
            } =>
                match inner_future.poll()? {
                    Async::NotReady =>
                        (
                            Async::NotReady,
                            UpgradeFutureState::WaitingOnInner {
                                has_result_cached,
                                inner_future,
                            },
                        ),
                    Async::Ready(CacheEntry::Cached(inner_object, meta)) => {
                        // TODO: figure out if this is really needed
                        futures::task::current().notify();
                        (
                            Async::NotReady,
                            UpgradeFutureState::Extending(meta, PendingUpgrade::new(inner_object, &self.gdcf, self.forced_refresh)?),
                        )
                    },
                    Async::Ready(cache_entry) => (Async::Ready(cache_entry.map_empty()), UpgradeFutureState::Exhausted),
                },

            UpgradeFutureState::Extending(meta, PendingUpgrade::Cached(object)) =>
                (Async::Ready(CacheEntry::Cached(object, meta)), UpgradeFutureState::Exhausted),

            UpgradeFutureState::Extending(meta, mut upgrade_mode) =>
                match upgrade_mode.future().unwrap().poll()? {
                    Async::NotReady => (Async::NotReady, UpgradeFutureState::Extending(meta, upgrade_mode)),
                    Async::Ready(cache_entry) =>
                        match upgrade_mode {
                            PendingUpgrade::Missing(to_upgrade, _) | PendingUpgrade::Outdated(to_upgrade, ..) => {
                                let upgrade = match cache_entry {
                                    CacheEntry::DeducedAbsent | CacheEntry::MarkedAbsent(_) =>
                                        U::default_upgrade().ok_or(Error::UnexpectedlyAbsent)?,
                                    CacheEntry::Cached(request_result, _) =>
                                        U::lookup_upgrade(&to_upgrade, &self.gdcf.cache(), request_result).map_err(Error::Cache)?,
                                    _ => unreachable!(),
                                };
                                let (upgraded, _) = to_upgrade.upgrade(upgrade);

                                (Async::Ready(CacheEntry::Cached(upgraded, meta)), UpgradeFutureState::Exhausted)
                            },
                            _ => unreachable!(),
                        },
                },

            UpgradeFutureState::Exhausted => panic!("Future already polled to completion"),
        };

        self.state = new_state;

        Ok(ready)
    }

    fn peek_cached<F: FnOnce(Self::GdcfItem) -> Self::GdcfItem>(self, f: F) -> Self {
        if !self.has_result_cached() {
            return self
        }

        let UpgradeFuture {
            state,
            gdcf,
            forced_refresh,
        } = self;

        let state = match state {
            UpgradeFutureState::WaitingOnInner {
                has_result_cached,
                inner_future,
            } =>
                UpgradeFutureState::WaitingOnInner {
                    has_result_cached,
                    inner_future: UpgradeFutureState::<From, Into, U>::peek_inner(&gdcf.cache(), inner_future, f),
                },
            UpgradeFutureState::Extending(meta, upgrade_mode) =>
                match upgrade_mode {
                    PendingUpgrade::Cached(cached) => UpgradeFutureState::Extending(meta, PendingUpgrade::Cached(f(cached))),
                    PendingUpgrade::Outdated(to_upgrade, upgrade, future) => {
                        let (upgraded, downgrade) = to_upgrade.upgrade(upgrade);
                        let (downgraded, upgrade) = U::downgrade(f(upgraded), downgrade);

                        UpgradeFutureState::Extending(meta, PendingUpgrade::Outdated(downgraded, upgrade, future))
                    },
                    PendingUpgrade::Missing(to_upgrade, future) =>
                        UpgradeFutureState::Extending(meta, PendingUpgrade::Missing(to_upgrade, future)),
                },
            UpgradeFutureState::Exhausted => UpgradeFutureState::Exhausted,
        };
        UpgradeFuture {
            state,
            gdcf,
            forced_refresh,
        }
    }

    fn is_absent(&self) -> bool {
        match self.state {
            UpgradeFutureState::WaitingOnInner { ref inner_future, .. } => inner_future.is_absent(),
            // if the inner future returns an absent variant the upgrade future resolves to that immediately and none of the other variants
            // are ever constructed
            _ => false,
        }
    }
}

impl<From, Into, U> CloneCached for UpgradeFuture<From, Into, U>
where
    From::ApiClient: MakeRequest<U::Request>,
    From::Cache: CanCache<U::Request>,
    From: GdcfFuture<GdcfItem = U> + CloneCached,
    U: Upgradable<From::Cache, Into> + Clone,
    Into: Clone,
    U::Upgrade: Clone,
{
    fn clone_cached(&self) -> Result<CacheEntry<Self::GdcfItem, <Self::Cache as Cache>::CacheEntryMeta>, ()> {
        match &self.state {
            UpgradeFutureState::WaitingOnInner { ref inner_future, .. } =>
                match inner_future.clone_cached()? {
                    CacheEntry::Cached(to_upgrade, meta) => {
                        let upgraded = temporary_upgrade(&inner_future.gdcf().cache(), to_upgrade).map_err(|_| ())?.0;

                        Ok(CacheEntry::Cached(upgraded, meta))
                    },
                    entry => Ok(entry.map_empty()),
                },
            UpgradeFutureState::Extending(meta, upgrade_mode) =>
                match upgrade_mode {
                    PendingUpgrade::Cached(cached) => Ok(CacheEntry::Cached(cached.clone(), *meta)),
                    PendingUpgrade::Outdated(to_upgrade, upgrade, _) =>
                        Ok(CacheEntry::Cached(to_upgrade.clone().upgrade(upgrade.clone()).0, *meta)),
                    PendingUpgrade::Missing(..) => Ok(CacheEntry::Missing),
                },
            UpgradeFutureState::Exhausted => Err(()),
        }
    }
}

impl<From, Into, U> Future for UpgradeFuture<From, Into, U>
where
    From::ApiClient: MakeRequest<U::Request>,
    From::Cache: CanCache<U::Request>,
    From: GdcfFuture<GdcfItem = U>,
    U: Upgradable<From::Cache, Into>,
{
    type Error = Error<<From::ApiClient as ApiClient>::Err, <From::Cache as Cache>::Err>;
    type Item = CacheEntry<Into, <From::Cache as Cache>::CacheEntryMeta>;

    fn poll(&mut self) -> Result<Async<Self::Item>, Self::Error> {
        GdcfFuture::poll(self)
    }
}

pub struct MultiUpgradeFuture<From, Into, U>
where
    From::ApiClient: MakeRequest<U::Request>,
    From::Cache: CanCache<U::Request>,
    From: GdcfFuture<GdcfItem = Vec<U>>,
    U: Upgradable<From::Cache, Into>,
{
    gdcf: Gdcf<From::ApiClient, From::Cache>,
    forced_refresh: bool,
    state: MultiUpgradeFutureState<From, Into, U>,
}

impl<From, Into, U> std::fmt::Debug for MultiUpgradeFuture<From, Into, U>
where
    From::ApiClient: MakeRequest<U::Request>,
    From::Cache: CanCache<U::Request>,
    From: GdcfFuture<GdcfItem = Vec<U>> + std::fmt::Debug,
    U: Upgradable<From::Cache, Into> + std::fmt::Debug,
    U::Upgrade: std::fmt::Debug,
    Into: std::fmt::Debug,
{
    fn fmt(&self, fmt: &mut std::fmt::Formatter) -> std::fmt::Result {
        fmt.debug_struct("MultiUpgradeFuture")
            .field("forced_refresh", &self.forced_refresh)
            .field("state", &self.state)
            .finish()
    }
}

impl<From, Into, U> MultiUpgradeFuture<From, Into, U>
where
    From::ApiClient: MakeRequest<U::Request>,
    From::Cache: CanCache<U::Request>,
    From: GdcfFuture<GdcfItem = Vec<U>>,
    U: Upgradable<From::Cache, Into>,
{
    pub fn upgrade_from(from: From) -> Self {
        let gdcf = from.gdcf();

        MultiUpgradeFuture {
            forced_refresh: from.forcing_refreshes(),
            state: MultiUpgradeFutureState::new(&gdcf.cache(), from),
            gdcf,
        }
    }

    pub fn upgrade_all<Into2>(self) -> MultiUpgradeFuture<Self, Into2, Into>
    where
        Into: Upgradable<From::Cache, Into2>,
        From::ApiClient: MakeRequest<Into::Request>,
        From::Cache: CanCache<Into::Request>,
    {
        MultiUpgradeFuture::upgrade_from(self)
    }
}

#[allow(missing_debug_implementations)]
enum MultiUpgradeFutureState<From, Into, U>
where
    From::ApiClient: MakeRequest<U::Request>,
    From::Cache: CanCache<U::Request>,
    From: GdcfFuture<GdcfItem = Vec<U>>,
    U: Upgradable<From::Cache, Into>,
{
    WaitingOnInner {
        has_result_cached: bool,
        inner_future: From,
    },
    Extending(
        <From::Cache as Cache>::CacheEntryMeta,
        Vec<PendingUpgrade<From::ApiClient, From::Cache, Into, U>>,
    ),
    Exhausted,
}

impl<From, Into, U> std::fmt::Debug for MultiUpgradeFutureState<From, Into, U>
where
    From::ApiClient: MakeRequest<U::Request>,
    From::Cache: CanCache<U::Request>,
    From: GdcfFuture<GdcfItem = Vec<U>> + std::fmt::Debug,
    U: Upgradable<From::Cache, Into> + std::fmt::Debug,
    U::Upgrade: std::fmt::Debug,
    Into: std::fmt::Debug,
{
    fn fmt(&self, fmt: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            MultiUpgradeFutureState::WaitingOnInner {
                has_result_cached,
                inner_future,
            } =>
                fmt.debug_struct("WaitingOnInner")
                    .field("has_result_cached", has_result_cached)
                    .field("inner_future", inner_future)
                    .finish(),
            MultiUpgradeFutureState::Extending(meta, upgrade_mode) => fmt.debug_tuple("Extending").field(meta).field(upgrade_mode).finish(),
            MultiUpgradeFutureState::Exhausted => fmt.debug_tuple("Exhausted").finish(),
        }
    }
}

impl<From, Into, U> MultiUpgradeFutureState<From, Into, U>
where
    From::ApiClient: MakeRequest<U::Request>,
    From::Cache: CanCache<U::Request>,
    From: GdcfFuture<GdcfItem = Vec<U>>,
    U: Upgradable<From::Cache, Into>,
{
    pub fn new(cache: &From::Cache, inner_future: From) -> Self {
        let mut has_result_cached = false;

        debug!("Constructing new MultiUpgradeFuture");

        MultiUpgradeFutureState::WaitingOnInner {
            inner_future: Self::peek_inner(cache, inner_future, |cached| {
                has_result_cached = true;
                cached
            }),
            has_result_cached,
        }
    }

    fn peek_inner(cache: &From::Cache, inner: From, f: impl FnOnce(Vec<Into>) -> Vec<Into>) -> From {
        trace!("MultiUpgradeFutureState::peek_inner called!");

        inner.peek_cached(move |e: Vec<U>| {
            trace!("inner.peek_cached closure called! We have {} elements", e.len());

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

impl<From, Into, U> GdcfFuture for MultiUpgradeFuture<From, Into, U>
where
    From::ApiClient: MakeRequest<U::Request>,
    From::Cache: CanCache<U::Request>,
    From: GdcfFuture<GdcfItem = Vec<U>>,
    U: Upgradable<From::Cache, Into>,
{
    type ApiClient = From::ApiClient;
    type BaseRequest = <From as GdcfFuture>::BaseRequest;
    type Cache = From::Cache;
    type GdcfItem = Vec<Into>;

    fn has_result_cached(&self) -> bool {
        match &self.state {
            MultiUpgradeFutureState::WaitingOnInner { has_result_cached, .. } => *has_result_cached,
            MultiUpgradeFutureState::Extending(_, upgrade_modes) =>
                upgrade_modes.iter().all(|mode| {
                    match mode {
                        PendingUpgrade::Cached(_) | PendingUpgrade::Outdated(..) => true,
                        PendingUpgrade::Missing(..) => false,
                    }
                }),
            MultiUpgradeFutureState::Exhausted => false,
        }
    }

    fn into_cached(
        self,
    ) -> Result<
        Result<CacheEntry<Self::GdcfItem, <Self::Cache as Cache>::CacheEntryMeta>, Self>,
        Error<<Self::ApiClient as ApiClient>::Err, <Self::Cache as Cache>::Err>,
    >
    where
        Self: Sized,
    {
        if !self.has_result_cached() {
            return Ok(Err(self))
        }

        match self.state {
            MultiUpgradeFutureState::WaitingOnInner { inner_future, .. } => {
                let cache = self.gdcf.cache();
                let base = match inner_future.into_cached()? {
                    Ok(base) =>
                        base.map(|to_upgrade| {
                            match temporary_upgrade_all(&cache, to_upgrade) {
                                Ok((upgraded, _)) => upgraded,
                                _ => unreachable!(),
                            }
                        }),
                    _ => unreachable!(),
                };

                Ok(Ok(base))
            },
            MultiUpgradeFutureState::Extending(meta, upgrade_modes) => {
                let mut result = Vec::new();

                for upgrade_mode in upgrade_modes {
                    match upgrade_mode {
                        PendingUpgrade::Cached(cached) => result.push(cached),
                        PendingUpgrade::Outdated(to_upgrade, upgrade, _) => result.push(to_upgrade.upgrade(upgrade).0),
                        PendingUpgrade::Missing(..) => unreachable!(),
                    }
                }

                Ok(Ok(CacheEntry::Cached(result, meta)))
            },
            MultiUpgradeFutureState::Exhausted => unreachable!(),
        }
    }

    fn new(gdcf: Gdcf<Self::ApiClient, Self::Cache>, request: &Self::BaseRequest) -> Result<Self, <Self::Cache as Cache>::Err> {
        Ok(MultiUpgradeFuture {
            forced_refresh: request.forces_refresh(),
            state: MultiUpgradeFutureState::new(&gdcf.cache(), From::new(gdcf.clone(), request)?),
            gdcf,
        })
    }

    fn peek_cached<F: FnOnce(Self::GdcfItem) -> Self::GdcfItem>(self, f: F) -> Self {
        if !self.has_result_cached() {
            return self
        }

        let MultiUpgradeFuture {
            state,
            gdcf,
            forced_refresh,
        } = self;

        let state = match state {
            MultiUpgradeFutureState::WaitingOnInner {
                has_result_cached,
                inner_future,
            } =>
                MultiUpgradeFutureState::WaitingOnInner {
                    has_result_cached,
                    inner_future: MultiUpgradeFutureState::<From, Into, U>::peek_inner(&gdcf.cache(), inner_future, f),
                },
            MultiUpgradeFutureState::Extending(meta, upgrade_modes) => {
                let mut upgraded = Vec::new();
                let mut downgrades = Vec::new();
                let mut futures = Vec::new();

                let mut failed = Vec::new();

                for upgrade_mode in upgrade_modes {
                    if !failed.is_empty() {
                        failed.push(upgrade_mode)
                    } else {
                        match upgrade_mode {
                            PendingUpgrade::Cached(cached) => {
                                upgraded.push(cached);
                                downgrades.push(None);
                                futures.push(None);
                            },
                            PendingUpgrade::Outdated(to_upgrade, upgrade, future) => {
                                let (is_upgraded, downgrade) = to_upgrade.upgrade(upgrade);

                                upgraded.push(is_upgraded);
                                downgrades.push(Some(downgrade));
                                futures.push(Some(future));
                            },
                            PendingUpgrade::Missing(..) => {
                                while !upgraded.is_empty() {
                                    let upgraded = upgraded.remove(0);
                                    let downgrade = downgrades.remove(0);
                                    let future = futures.remove(0);

                                    failed.push(match downgrade {
                                        None => PendingUpgrade::Cached(upgraded),
                                        Some(downgrade) => {
                                            let (downgraded, upgrade) = U::downgrade(upgraded, downgrade);

                                            PendingUpgrade::Outdated(downgraded, upgrade, future.unwrap())
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
                                None => PendingUpgrade::Cached(upgraded),
                                Some(downgrade) => {
                                    let (downgraded, upgrade) = U::downgrade(upgraded, downgrade);

                                    PendingUpgrade::Outdated(downgraded, upgrade, future.unwrap())
                                },
                            }
                        })
                        .collect()
                } else {
                    failed
                };

                MultiUpgradeFutureState::Extending(meta, upgrade_modes)
            },
            MultiUpgradeFutureState::Exhausted => MultiUpgradeFutureState::Exhausted,
        };

        MultiUpgradeFuture {
            state,
            gdcf,
            forced_refresh,
        }
    }

    fn gdcf(&self) -> Gdcf<Self::ApiClient, Self::Cache> {
        self.gdcf.clone()
    }

    fn forcing_refreshes(&self) -> bool {
        self.forced_refresh
    }

    fn poll(
        &mut self,
    ) -> Result<
        Async<CacheEntry<Self::GdcfItem, <Self::Cache as Cache>::CacheEntryMeta>>,
        Error<<Self::ApiClient as ApiClient>::Err, <Self::Cache as Cache>::Err>,
    > {
        let (ready, new_state) = match std::mem::replace(&mut self.state, MultiUpgradeFutureState::Exhausted) {
            MultiUpgradeFutureState::WaitingOnInner {
                has_result_cached,
                mut inner_future,
            } => {
                match inner_future.poll()? {
                    Async::NotReady =>
                        (
                            Async::NotReady,
                            MultiUpgradeFutureState::WaitingOnInner {
                                has_result_cached,
                                inner_future,
                            },
                        ),
                    Async::Ready(CacheEntry::Cached(cached_objects, meta)) => {
                        // TODO: figure out if this is really needed
                        futures::task::current().notify();

                        (
                            Async::NotReady,
                            MultiUpgradeFutureState::Extending(
                                meta,
                                cached_objects
                                    .into_iter()
                                    .map(|object| PendingUpgrade::new(object, &self.gdcf, self.forced_refresh))
                                    .collect::<Result<Vec<_>, _>>()?,
                            ),
                        )
                    },
                    Async::Ready(cache_entry) => (Async::Ready(cache_entry.map_empty()), MultiUpgradeFutureState::Exhausted),
                }
            },

            MultiUpgradeFutureState::Extending(meta, entry_upgrade_modes) => {
                let mut done = Vec::new();
                let mut not_done = Vec::new();

                for upgrade_mode in entry_upgrade_modes {
                    match upgrade_mode {
                        PendingUpgrade::Cached(cached) => done.push(cached),
                        mut upgrade_mode =>
                            match upgrade_mode.future().unwrap().poll()? {
                                Async::NotReady => not_done.push(upgrade_mode),
                                Async::Ready(cache_entry) => {
                                    let to_upgrade = upgrade_mode.into_upgradable().unwrap();
                                    let upgrade = match cache_entry {
                                        CacheEntry::MarkedAbsent(_) | CacheEntry::DeducedAbsent =>
                                            U::default_upgrade().ok_or(Error::UnexpectedlyAbsent)?,
                                        CacheEntry::Cached(request_result, _) =>
                                            U::lookup_upgrade(&to_upgrade, &self.gdcf.cache(), request_result).map_err(Error::Cache)?,
                                        _ => unreachable!(),
                                    };
                                    let (upgraded, _) = to_upgrade.upgrade(upgrade);

                                    done.push(upgraded);
                                },
                            },
                    }
                }

                if not_done.is_empty() {
                    (Async::Ready(CacheEntry::Cached(done, meta)), MultiUpgradeFutureState::Exhausted)
                } else {
                    not_done.extend(done.into_iter().map(PendingUpgrade::Cached));
                    (Async::NotReady, MultiUpgradeFutureState::Extending(meta, not_done))
                }
            },

            MultiUpgradeFutureState::Exhausted => panic!("Future already polled to completion"),
        };

        self.state = new_state;

        Ok(ready)
    }

    fn is_absent(&self) -> bool {
        match self.state {
            MultiUpgradeFutureState::WaitingOnInner { ref inner_future, .. } => inner_future.is_absent(),
            // if the inner future returns an absent variant the upgrade future resolves to that immediately and none of the other variants
            // are ever constructed
            _ => false,
        }
    }
}

impl<From, Into, U> CloneCached for MultiUpgradeFuture<From, Into, U>
where
    From::ApiClient: MakeRequest<U::Request>,
    From::Cache: CanCache<U::Request>,
    From: GdcfFuture<GdcfItem = Vec<U>> + CloneCached,
    U: Upgradable<From::Cache, Into> + Clone,
    Into: Clone,
    U::Upgrade: Clone,
{
    fn clone_cached(&self) -> Result<CacheEntry<Self::GdcfItem, <Self::Cache as Cache>::CacheEntryMeta>, ()> {
        match &self.state {
            MultiUpgradeFutureState::WaitingOnInner { ref inner_future, .. } =>
                match inner_future.clone_cached()? {
                    CacheEntry::Cached(to_upgrade, meta) => {
                        let upgraded = temporary_upgrade_all(&inner_future.gdcf().cache(), to_upgrade).map_err(|_| ())?.0;

                        Ok(CacheEntry::Cached(upgraded, meta))
                    },
                    entry => Ok(entry.map_empty()),
                },
            MultiUpgradeFutureState::Extending(ref meta, ref upgrade_modes) =>
                if !self.has_result_cached() {
                    Ok(CacheEntry::Missing)
                } else {
                    let mut clones = Vec::new();

                    for mode in upgrade_modes {
                        match mode {
                            PendingUpgrade::Cached(cached) => clones.push(cached.clone()),
                            PendingUpgrade::Outdated(to_upgrade, upgrade, _) => clones.push(to_upgrade.clone().upgrade(upgrade.clone()).0),
                            PendingUpgrade::Missing(..) => unreachable!(),
                        }
                    }

                    Ok(CacheEntry::Cached(clones, *meta))
                },
            MultiUpgradeFutureState::Exhausted => Err(()),
        }
    }
}

impl<From, Into, U> Future for MultiUpgradeFuture<From, Into, U>
where
    From::ApiClient: MakeRequest<U::Request>,
    From::Cache: CanCache<U::Request>,
    From: GdcfFuture<GdcfItem = Vec<U>>,
    U: Upgradable<From::Cache, Into>,
{
    type Error = Error<<From::ApiClient as ApiClient>::Err, <From::Cache as Cache>::Err>;
    type Item = CacheEntry<Vec<Into>, <From::Cache as Cache>::CacheEntryMeta>;

    fn poll(&mut self) -> Result<Async<Self::Item>, Self::Error> {
        GdcfFuture::poll(self)
    }
}

fn temporary_upgrade<C: Cache + CanCache<U::Request>, Into, U: Upgradable<C, Into>>(
    cache: &C,
    to_upgrade: U,
) -> Result<(Into, U::From), U> {
    let upgrade = match U::upgrade_request(&to_upgrade) {
        Some(request) =>
            match cache.lookup_request(&request) {
                Ok(CacheEntry::Cached(cached_result, _)) =>
                    match U::lookup_upgrade(&to_upgrade, cache, cached_result) {
                        Ok(upgrade) => upgrade,
                        _ => {
                            error!("Error on internal cache lookup in termporary_upgrade");

                            return Err(to_upgrade)
                        },
                    },
                Ok(CacheEntry::Missing) => return Err(to_upgrade), // no information about the upgrade was cached
                Ok(_) =>
                    match U::default_upgrade() {
                        // upgrade was marked/deduced as absent
                        Some(upgrade) => upgrade,
                        _ => {
                            error!("Error on internal cache lookup in termporary_upgrade");

                            return Err(to_upgrade)
                        },
                    },
                _ => {
                    error!("Error on internal cache lookup in termporary_upgrade");

                    return Err(to_upgrade)
                }, // error during request lookup
            },
        None =>
            match U::default_upgrade() {
                Some(upgrade) => upgrade,
                _ => {
                    /* no upgrade request and no default (incorrect Upgrade impl, but that's something we check
                     * later) */
                    warn!("Default upgraded unexpectedly not available for absent data.");

                    return Err(to_upgrade)
                },
            },
    };

    Ok(to_upgrade.upgrade(upgrade))
}

fn temporary_upgrade_all<C: Cache + CanCache<U::Request>, Into, U: Upgradable<C, Into>>(
    cache: &C,
    to_upgrade: Vec<U>,
) -> Result<(Vec<Into>, Vec<U::From>), Vec<U>> {
    let mut temporarily_upgraded = Vec::new();
    let mut downgrades = Vec::new();

    let mut failed = Vec::new();

    let before = to_upgrade.len();

    trace!("temporary_upgrade_all called. We have {} elements", before);

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

    assert_eq!(before, failed.len() + temporarily_upgraded.len());
    assert_eq!(before, failed.len() + downgrades.len());
    assert_eq!(downgrades.len(), temporarily_upgraded.len());

    if failed.is_empty() {
        trace!(
            "temporary_upgrade_all done. We have upgraded {} elements",
            temporarily_upgraded.len()
        );
        Ok((temporarily_upgraded, downgrades))
    } else {
        trace!("temporary_upgrade_all failed. We have {} failed elements", failed.len());
        Err(failed)
    }
}
