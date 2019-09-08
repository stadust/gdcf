use crate::{
    api::{client::MakeRequest, request::Request, ApiClient},
    cache::{Cache, CacheEntry, CanCache, Store},
    error::GdcfError,
    future::{
        refresh::RefreshCacheFuture,
        upgrade::{MultiUpgradeFuture, UpgradeFuture},
        GdcfFuture,
    },
    upgrade::Upgrade,
    Gdcf,
};
use futures::{Async, Future};
use gdcf_model::{song::NewgroundsSong, user::Creator};

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

pub(crate) enum ProcessRequestFutureState<Req, A, C>
where
    A: ApiClient + MakeRequest<Req>,
    C: Cache + Store<Creator> + Store<NewgroundsSong> + CanCache<Req>,
    Req: Request,
{
    Empty,
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
            ProcessRequestFutureState::Empty => fmt.debug_tuple("Empty").finish(),
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
        Req::Result: Upgrade<C, Into>,
        A: MakeRequest<<Req::Result as Upgrade<C, Into>>::Request>,
        C: CanCache<<Req::Result as Upgrade<C, Into>>::Request>,
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
        T: Upgrade<C, Into>,
        A: MakeRequest<<T as Upgrade<C, Into>>::Request>,
        C: CanCache<<T as Upgrade<C, Into>>::Request>,
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
    type Error = GdcfError<A::Err, C::Err>;
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

    fn into_cached(
        self,
    ) -> Result<Result<CacheEntry<Self::GdcfItem, <Self::Cache as Cache>::CacheEntryMeta>, Self>, GdcfError<A::Err, C::Err>>
    where
        Self: Sized,
    {
        match self.state {
            ProcessRequestFutureState::Empty | ProcessRequestFutureState::Uncached(_) => Ok(Err(self)),
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

        let state = match state {
            ProcessRequestFutureState::Outdated(CacheEntry::Cached(object, meta), future) =>
                ProcessRequestFutureState::Outdated(CacheEntry::Cached(f(object), meta), future),
            ProcessRequestFutureState::UpToDate(CacheEntry::Cached(object, meta)) =>
                ProcessRequestFutureState::UpToDate(CacheEntry::Cached(f(object), meta)),
            _ => state,
        };

        ProcessRequestFuture {
            state,
            gdcf,
            forces_refresh,
        }
    }

    fn gdcf(&self) -> Gdcf<Self::ApiClient, Self::Cache> {
        self.gdcf.clone()
    }

    fn forcing_refreshs(&self) -> bool {
        self.forces_refresh
    }

    fn poll(
        &mut self,
    ) -> Result<
        Async<CacheEntry<Self::GdcfItem, <Self::Cache as Cache>::CacheEntryMeta>>,
        GdcfError<<Self::ApiClient as ApiClient>::Err, <Self::Cache as Cache>::Err>,
    > {
        match &mut self.state {
            ProcessRequestFutureState::Empty => panic!("Future already polled to completion"),
            ProcessRequestFutureState::Uncached(future) => future.poll(),
            ProcessRequestFutureState::Outdated(_, future) => future.poll(),
            fut =>
                match std::mem::replace(fut, ProcessRequestFutureState::Empty) {
                    ProcessRequestFutureState::UpToDate(inner) => Ok(Async::Ready(inner)),
                    _ => unreachable!(),
                },
        }
    }
}
