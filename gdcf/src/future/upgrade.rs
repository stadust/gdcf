use futures::{Async, Future};

use crate::{
    api::{client::MakeRequest, request::Request, ApiClient},
    cache::{Cache, CacheEntry, CanCache},
    error::GdcfError,
    future::GdcfFuture,
    upgrade::{Upgrade, UpgradeMode},
    Gdcf,
};

#[allow(missing_debug_implementations)]
pub struct UpgradeFuture<From, Into, U>
where
    From::ApiClient: MakeRequest<U::Request>,
    From::Cache: CanCache<U::Request>,
    From: GdcfFuture<GdcfItem = U>,
    U: Upgrade<From::Cache, Into>,
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
    U: Upgrade<From::Cache, Into> + std::fmt::Debug,
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
    U: Upgrade<From::Cache, Into>,
{
    pub fn upgrade_from(from: From) -> Self {
        let gdcf = from.gdcf();

        UpgradeFuture {
            forced_refresh: from.forcing_refreshs(),
            state: UpgradeFutureState::new(&gdcf.cache(), from),
            gdcf,
        }
    }

    pub fn upgrade<Into2>(self) -> UpgradeFuture<Self, Into2, Into>
    where
        Into: Upgrade<From::Cache, Into2>,
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
    U: Upgrade<From::Cache, Into>,
{
    WaitingOnInner {
        has_result_cached: bool,
        inner_future: From,
    },
    Extending(
        <From::Cache as Cache>::CacheEntryMeta,
        UpgradeMode<From::ApiClient, From::Cache, Into, U>,
    ),
    Exhausted,
}

impl<From, Into, U> std::fmt::Debug for UpgradeFutureState<From, Into, U>
where
    From::ApiClient: MakeRequest<U::Request>,
    From::Cache: CanCache<U::Request>,
    From: GdcfFuture<GdcfItem = U> + std::fmt::Debug,
    U: Upgrade<From::Cache, Into> + std::fmt::Debug,
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
    U: Upgrade<From::Cache, Into>,
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
    U: Upgrade<From::Cache, Into>,
{
    type ApiClient = From::ApiClient;
    type BaseRequest = <From as GdcfFuture>::BaseRequest;
    type Cache = From::Cache;
    type GdcfItem = Into;

    fn has_result_cached(&self) -> bool {
        match &self.state {
            UpgradeFutureState::WaitingOnInner { has_result_cached, .. } => *has_result_cached,
            UpgradeFutureState::Extending(_, upgrade_mode) =>
                match upgrade_mode {
                    UpgradeMode::UpgradeCached(_) | UpgradeMode::UpgradeOutdated(..) => true,
                    UpgradeMode::UpgradeMissing(..) => false,
                },
            UpgradeFutureState::Exhausted => false,
        }
    }

    fn into_cached(
        self,
    ) -> Result<
        Result<CacheEntry<Self::GdcfItem, <Self::Cache as Cache>::CacheEntryMeta>, Self>,
        GdcfError<<Self::ApiClient as ApiClient>::Err, <Self::Cache as Cache>::Err>,
    >
    where
        Self: Sized,
    {
        if !self.has_result_cached() {
            return Ok(Err(self))
        }

        match self.state {
            UpgradeFutureState::WaitingOnInner { inner_future, .. } => {
                let base = match inner_future.into_cached()? {
                    Ok(base) => base,
                    _ => unreachable!(),
                };

                Ok(Ok(match base {
                    CacheEntry::Cached(to_upgrade, meta) =>
                        CacheEntry::Cached(
                            match temporary_upgrade(&self.gdcf.cache(), to_upgrade) {
                                Ok((upgraded, _)) => upgraded,
                                _ => unreachable!(),
                            },
                            meta,
                        ),
                    cache_entry => cache_entry.map_empty(),
                }))
            },
            UpgradeFutureState::Extending(meta, upgrade_mode) =>
                match upgrade_mode {
                    UpgradeMode::UpgradeCached(cached) => Ok(Ok(CacheEntry::Cached(cached, meta))),
                    UpgradeMode::UpgradeOutdated(to_upgrade, upgrade, _) => Ok(Ok(CacheEntry::Cached(to_upgrade.upgrade(upgrade).0, meta))),
                    UpgradeMode::UpgradeMissing(..) => unreachable!(),
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

    fn forcing_refreshs(&self) -> bool {
        self.forced_refresh
    }

    fn poll(
        &mut self,
    ) -> Result<
        Async<CacheEntry<Self::GdcfItem, <Self::Cache as Cache>::CacheEntryMeta>>,
        GdcfError<<Self::ApiClient as ApiClient>::Err, <Self::Cache as Cache>::Err>,
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
                            UpgradeFutureState::Extending(meta, UpgradeMode::new(inner_object, &self.gdcf, self.forced_refresh)?),
                        )
                    },
                    Async::Ready(cache_entry) => (Async::Ready(cache_entry.map_empty()), UpgradeFutureState::Exhausted),
                },

            UpgradeFutureState::Extending(meta, UpgradeMode::UpgradeCached(object)) =>
                (Async::Ready(CacheEntry::Cached(object, meta)), UpgradeFutureState::Exhausted),

            UpgradeFutureState::Extending(meta, mut upgrade_mode) =>
                match upgrade_mode.future().unwrap().poll()? {
                    Async::NotReady => (Async::NotReady, UpgradeFutureState::Extending(meta, upgrade_mode)),
                    Async::Ready(cache_entry) =>
                        match upgrade_mode {
                            UpgradeMode::UpgradeMissing(to_upgrade, _) | UpgradeMode::UpgradeOutdated(to_upgrade, ..) => {
                                let upgrade = match cache_entry {
                                    CacheEntry::DeducedAbsent | CacheEntry::MarkedAbsent(_) =>
                                        U::default_upgrade().ok_or(GdcfError::ConsistencyAssumptionViolated)?,
                                    CacheEntry::Cached(request_result, _) =>
                                        U::lookup_upgrade(&to_upgrade, &self.gdcf.cache(), request_result).map_err(GdcfError::Cache)?,
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
                    UpgradeMode::UpgradeCached(cached) => UpgradeFutureState::Extending(meta, UpgradeMode::UpgradeCached(f(cached))),
                    UpgradeMode::UpgradeOutdated(to_upgrade, upgrade, future) => {
                        let (upgraded, downgrade) = to_upgrade.upgrade(upgrade);
                        let (downgraded, upgrade) = U::downgrade(f(upgraded), downgrade);

                        UpgradeFutureState::Extending(meta, UpgradeMode::UpgradeOutdated(downgraded, upgrade, future))
                    },
                    UpgradeMode::UpgradeMissing(to_upgrade, future) =>
                        UpgradeFutureState::Extending(meta, UpgradeMode::UpgradeMissing(to_upgrade, future)),
                },
            UpgradeFutureState::Exhausted => UpgradeFutureState::Exhausted,
        };
        UpgradeFuture {
            state,
            gdcf,
            forced_refresh,
        }
    }
}

impl<From, Into, U> Future for UpgradeFuture<From, Into, U>
where
    From::ApiClient: MakeRequest<U::Request>,
    From::Cache: CanCache<U::Request>,
    From: GdcfFuture<GdcfItem = U>,
    U: Upgrade<From::Cache, Into>,
{
    type Error = GdcfError<<From::ApiClient as ApiClient>::Err, <From::Cache as Cache>::Err>;
    type Item = CacheEntry<Into, <From::Cache as Cache>::CacheEntryMeta>;

    fn poll(&mut self) -> Result<Async<Self::Item>, Self::Error> {
        GdcfFuture::poll(self)
    }
}

#[allow(missing_debug_implementations)]
pub struct MultiUpgradeFuture<From, Into, U>
where
    From::ApiClient: MakeRequest<U::Request>,
    From::Cache: CanCache<U::Request>,
    From: GdcfFuture<GdcfItem = Vec<U>>,
    U: Upgrade<From::Cache, Into>,
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
    U: Upgrade<From::Cache, Into> + std::fmt::Debug,
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
    U: Upgrade<From::Cache, Into>,
{
    pub fn upgrade_from(from: From) -> Self {
        let gdcf = from.gdcf();

        MultiUpgradeFuture {
            forced_refresh: from.forcing_refreshs(),
            state: MultiUpgradeFutureState::new(&gdcf.cache(), from),
            gdcf,
        }
    }

    pub fn upgrade_all<Into2>(self) -> MultiUpgradeFuture<Self, Into2, Into>
    where
        Into: Upgrade<From::Cache, Into2>,
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
    U: Upgrade<From::Cache, Into>,
{
    WaitingOnInner {
        has_result_cached: bool,
        inner_future: From,
    },
    Extending(
        <From::Cache as Cache>::CacheEntryMeta,
        Vec<UpgradeMode<From::ApiClient, From::Cache, Into, U>>,
    ),
    Exhausted,
}

impl<From, Into, U> std::fmt::Debug for MultiUpgradeFutureState<From, Into, U>
where
    From::ApiClient: MakeRequest<U::Request>,
    From::Cache: CanCache<U::Request>,
    From: GdcfFuture<GdcfItem = Vec<U>> + std::fmt::Debug,
    U: Upgrade<From::Cache, Into> + std::fmt::Debug,
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
    U: Upgrade<From::Cache, Into>,
{
    pub fn new(cache: &From::Cache, inner_future: From) -> Self {
        let mut has_result_cached = false;

        MultiUpgradeFutureState::WaitingOnInner {
            inner_future: Self::peek_inner(cache, inner_future, |cached| {
                has_result_cached = true;
                cached
            }),
            has_result_cached,
        }
    }

    fn peek_inner(cache: &From::Cache, inner: From, f: impl FnOnce(Vec<Into>) -> Vec<Into>) -> From {
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

impl<From, Into, U> GdcfFuture for MultiUpgradeFuture<From, Into, U>
where
    From::ApiClient: MakeRequest<U::Request>,
    From::Cache: CanCache<U::Request>,
    From: GdcfFuture<GdcfItem = Vec<U>>,
    U: Upgrade<From::Cache, Into>,
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
                        UpgradeMode::UpgradeCached(_) | UpgradeMode::UpgradeOutdated(..) => true,
                        UpgradeMode::UpgradeMissing(..) => false,
                    }
                }),
            MultiUpgradeFutureState::Exhausted => false,
        }
    }

    fn into_cached(
        self,
    ) -> Result<
        Result<CacheEntry<Self::GdcfItem, <Self::Cache as Cache>::CacheEntryMeta>, Self>,
        GdcfError<<Self::ApiClient as ApiClient>::Err, <Self::Cache as Cache>::Err>,
    >
    where
        Self: Sized,
    {
        if !self.has_result_cached() {
            return Ok(Err(self))
        }

        match self.state {
            MultiUpgradeFutureState::WaitingOnInner { inner_future, .. } => {
                let base = match inner_future.into_cached()? {
                    Ok(base) => base,
                    _ => unreachable!(),
                };

                Ok(Ok(match base {
                    CacheEntry::Cached(to_upgrade, meta) =>
                        CacheEntry::Cached(
                            match temporary_upgrade_all(&self.gdcf.cache(), to_upgrade) {
                                Ok((upgraded, _)) => upgraded,
                                _ => unreachable!(),
                            },
                            meta,
                        ),
                    cache_entry => cache_entry.map_empty(),
                }))
            },
            MultiUpgradeFutureState::Extending(meta, upgrade_modes) => {
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

    fn forcing_refreshs(&self) -> bool {
        self.forced_refresh
    }

    fn poll(
        &mut self,
    ) -> Result<
        Async<CacheEntry<Self::GdcfItem, <Self::Cache as Cache>::CacheEntryMeta>>,
        GdcfError<<Self::ApiClient as ApiClient>::Err, <Self::Cache as Cache>::Err>,
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
                                    .map(|object| UpgradeMode::new(object, &self.gdcf, self.forced_refresh))
                                    .collect::<Result<Vec<_>, _>>()?,
                            ),
                        )
                    },
                    Async::Ready(cache_entry) => (Async::Ready(cache_entry.map_empty()), MultiUpgradeFutureState::Exhausted),
                }
            },

            MultiUpgradeFutureState::Extending(meta, mut entry_upgrade_modes) => {
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
                                            U::lookup_upgrade(&to_upgrade, &self.gdcf.cache(), request_result).map_err(GdcfError::Cache)?,
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
                    not_done.extend(done.into_iter().map(UpgradeMode::UpgradeCached));
                    (Async::NotReady, MultiUpgradeFutureState::Extending(meta, not_done))
                }
            },

            MultiUpgradeFutureState::Exhausted => panic!("Future already polled to completion"),
        };

        self.state = new_state;

        Ok(ready)
    }
}

impl<From, Into, U> Future for MultiUpgradeFuture<From, Into, U>
where
    From::ApiClient: MakeRequest<U::Request>,
    From::Cache: CanCache<U::Request>,
    From: GdcfFuture<GdcfItem = Vec<U>>,
    U: Upgrade<From::Cache, Into>,
{
    type Error = GdcfError<<From::ApiClient as ApiClient>::Err, <From::Cache as Cache>::Err>;
    type Item = CacheEntry<Vec<Into>, <From::Cache as Cache>::CacheEntryMeta>;

    fn poll(&mut self) -> Result<Async<Self::Item>, Self::Error> {
        GdcfFuture::poll(self)
    }
}

fn temporary_upgrade<C: Cache + CanCache<U::Request>, Into, U: Upgrade<C, Into>>(cache: &C, to_upgrade: U) -> Result<(Into, U::From), U> {
    let upgrade = match U::upgrade_request(&to_upgrade) {
        Some(request) =>
            match cache.lookup_request(&request) {
                Ok(CacheEntry::Cached(cached_result, _)) =>
                    match U::lookup_upgrade(&to_upgrade, cache, cached_result) {
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
