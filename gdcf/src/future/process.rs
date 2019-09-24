use futures::{Async, Future};
use log::trace;

use gdcf_model::{song::NewgroundsSong, user::Creator};

use crate::{
    api::{client::MakeRequest, request::Request, ApiClient},
    cache::{Cache, CacheEntry, CanCache, Store},
    error::Error,
    future::{
        refresh::RefreshCacheFuture,
        upgrade::{MultiUpgradeFuture, UpgradeFuture},
        CloneCached, GdcfFuture,
    },
    upgrade::Upgradable,
    Gdcf,
};

pub struct ProcessRequestFuture<Req, A, C>
where
    A: ApiClient + MakeRequest<Req>,
    C: Cache + Store<Creator> + Store<NewgroundsSong> + CanCache<Req>,
    Req: Request,
{
    gdcf: Gdcf<A, C>,
    forces_refresh: bool,
    state: ProcessRequestFutureState<Req, A, C>,
}

impl<Req, A, C> std::fmt::Debug for ProcessRequestFuture<Req, A, C>
where
    A: ApiClient + MakeRequest<Req>,
    C: Cache + Store<Creator> + Store<NewgroundsSong> + CanCache<Req>,
    Req: Request + std::fmt::Debug,
{
    fn fmt(&self, fmt: &mut std::fmt::Formatter) -> std::fmt::Result {
        fmt.debug_struct("ProcessRequestFuture")
            .field("forces_refresh", &self.forces_refresh)
            .field("state", &self.state)
            .finish()
    }
}

// FIXME: this enum is incredibly similar to upgrade::PendingUpgrade. We might be able to use just
// that
pub(crate) enum ProcessRequestFutureState<Req, A, C>
where
    A: ApiClient + MakeRequest<Req>,
    C: Cache + Store<Creator> + Store<NewgroundsSong> + CanCache<Req>,
    Req: Request,
{
    Exhausted,
    Uncached(RefreshCacheFuture<Req, A, C>),
    Outdated(CacheEntry<Req::Result, C::CacheEntryMeta>, RefreshCacheFuture<Req, A, C>),
    UpToDate(CacheEntry<Req::Result, C::CacheEntryMeta>),
}

impl<Req, A, C> std::fmt::Debug for ProcessRequestFutureState<Req, A, C>
where
    A: ApiClient + MakeRequest<Req>,
    C: Cache + Store<Creator> + Store<NewgroundsSong> + CanCache<Req>,
    Req: Request,
{
    fn fmt(&self, fmt: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            ProcessRequestFutureState::Exhausted => fmt.debug_tuple("Empty").finish(),
            ProcessRequestFutureState::Uncached(fut) => fmt.debug_tuple("Uncached").field(fut).finish(),
            ProcessRequestFutureState::Outdated(cached, fut) => fmt.debug_tuple("Outdated").field(cached).field(fut).finish(),
            ProcessRequestFutureState::UpToDate(cached) => fmt.debug_tuple("UpToDate").field(cached).finish(),
        }
    }
}

impl<Req, A, C> ProcessRequestFuture<Req, A, C>
where
    A: ApiClient + MakeRequest<Req>,
    C: Cache + Store<Creator> + Store<NewgroundsSong> + CanCache<Req>,
    Req: Request,
{
    pub fn upgrade<Into>(self) -> UpgradeFuture<Self, Into, Req::Result>
    where
        Req::Result: Upgradable<C, Into>,
        A: MakeRequest<<Req::Result as Upgradable<C, Into>>::Request>,
        C: CanCache<<Req::Result as Upgradable<C, Into>>::Request>,
    {
        UpgradeFuture::upgrade_from(self)
    }
}

impl<Req, A, C, T> ProcessRequestFuture<Req, A, C>
where
    A: ApiClient + MakeRequest<Req>,
    C: Cache + Store<Creator> + Store<NewgroundsSong> + CanCache<Req>,
    Req: Request<Result = Vec<T>>,
    T: std::fmt::Debug + Send + Sync + 'static,
{
    pub fn upgrade_all<Into>(self) -> MultiUpgradeFuture<Self, Into, T>
    where
        T: Upgradable<C, Into>,
        A: MakeRequest<<T as Upgradable<C, Into>>::Request>,
        C: CanCache<<T as Upgradable<C, Into>>::Request>,
    {
        MultiUpgradeFuture::upgrade_from(self)
    }
}

impl<Req, A, C> Future for ProcessRequestFuture<Req, A, C>
where
    A: ApiClient + MakeRequest<Req>,
    C: Cache + Store<Creator> + Store<NewgroundsSong> + CanCache<Req>,
    Req: Request,
{
    type Error = Error<A::Err, C::Err>;
    type Item = CacheEntry<Req::Result, C::CacheEntryMeta>;

    fn poll(&mut self) -> Result<Async<Self::Item>, Self::Error> {
        GdcfFuture::poll(self)
    }
}

impl<Req, A, C> GdcfFuture for ProcessRequestFuture<Req, A, C>
where
    A: ApiClient + MakeRequest<Req>,
    C: Cache + Store<Creator> + Store<NewgroundsSong> + CanCache<Req>,
    Req: Request,
{
    type ApiClient = A;
    type BaseRequest = Req;
    type Cache = C;
    type GdcfItem = Req::Result;

    fn has_result_cached(&self) -> bool {
        match self.state {
            ProcessRequestFutureState::Outdated(..) | ProcessRequestFutureState::UpToDate(..) => true,
            _ => false,
        }
    }

    fn into_cached(self) -> Result<Result<CacheEntry<Self::GdcfItem, <Self::Cache as Cache>::CacheEntryMeta>, Self>, Error<A::Err, C::Err>>
    where
        Self: Sized,
    {
        match self.state {
            ProcessRequestFutureState::Exhausted | ProcessRequestFutureState::Uncached(_) => Ok(Err(self)),
            ProcessRequestFutureState::Outdated(cache_entry, _) | ProcessRequestFutureState::UpToDate(cache_entry) => Ok(Ok(cache_entry)),
        }
    }

    fn new(gdcf: Gdcf<A, C>, request: &Self::BaseRequest) -> Result<Self, C::Err> {
        Ok(ProcessRequestFuture {
            forces_refresh: request.forces_refresh(),
            state: gdcf.process(request)?,
            gdcf,
        })
    }

    fn peek_cached<F: FnOnce(Self::GdcfItem) -> Self::GdcfItem>(self, f: F) -> Self {
        let ProcessRequestFuture {
            gdcf,
            forces_refresh,
            state,
        } = self;

        trace!("State before executing peek_cached closure: {:?}", state);

        let state = match state {
            ProcessRequestFutureState::Outdated(CacheEntry::Cached(object, meta), future) =>
                ProcessRequestFutureState::Outdated(CacheEntry::Cached(f(object), meta), future),
            ProcessRequestFutureState::UpToDate(CacheEntry::Cached(object, meta)) =>
                ProcessRequestFutureState::UpToDate(CacheEntry::Cached(f(object), meta)),
            _ => state,
        };

        trace!("State after executing peek_cached closure: {:?}", state);

        ProcessRequestFuture {
            state,
            gdcf,
            forces_refresh,
        }
    }

    fn gdcf(&self) -> Gdcf<Self::ApiClient, Self::Cache> {
        self.gdcf.clone()
    }

    fn forcing_refreshes(&self) -> bool {
        self.forces_refresh
    }

    fn poll(
        &mut self,
    ) -> Result<
        Async<CacheEntry<Self::GdcfItem, <Self::Cache as Cache>::CacheEntryMeta>>,
        Error<<Self::ApiClient as ApiClient>::Err, <Self::Cache as Cache>::Err>,
    > {
        match &mut self.state {
            ProcessRequestFutureState::Exhausted => panic!("Future already polled to completion"),
            ProcessRequestFutureState::Uncached(future) => future.poll(),
            ProcessRequestFutureState::Outdated(_, future) => future.poll(),
            fut =>
                match std::mem::replace(fut, ProcessRequestFutureState::Exhausted) {
                    ProcessRequestFutureState::UpToDate(inner) => Ok(Async::Ready(inner)),
                    _ => unreachable!(),
                },
        }
    }

    fn is_absent(&self) -> bool {
        match self.state {
            ProcessRequestFutureState::Outdated(CacheEntry::DeducedAbsent, _)
            | ProcessRequestFutureState::Outdated(CacheEntry::MarkedAbsent(_), _)
            | ProcessRequestFutureState::UpToDate(CacheEntry::DeducedAbsent)
            | ProcessRequestFutureState::UpToDate(CacheEntry::MarkedAbsent(_)) => true,
            _ => false,
        }
    }
}

impl<Req, A, C> CloneCached for ProcessRequestFuture<Req, A, C>
where
    A: ApiClient + MakeRequest<Req>,
    C: Cache + Store<Creator> + Store<NewgroundsSong> + CanCache<Req>,
    Req: Request,
    Req::Result: Clone,
{
    fn clone_cached(&self) -> Result<CacheEntry<Self::GdcfItem, <Self::Cache as Cache>::CacheEntryMeta>, ()> {
        match &self.state {
            ProcessRequestFutureState::Exhausted => Err(()),
            ProcessRequestFutureState::Uncached(_) => Ok(CacheEntry::Missing),
            ProcessRequestFutureState::Outdated(cached, _) | ProcessRequestFutureState::UpToDate(cached) => Ok(cached.clone()),
        }
    }
}
