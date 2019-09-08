use crate::{
    api::{
        client::MakeRequest,
        request::{PaginatableRequest, Request},
        ApiClient,
    },
    cache::{Cache, CacheEntry, CanCache, Store},
    error::{ApiError, GdcfError},
    future::{
        refresh::RefreshCacheFuture,
        upgrade::{MultiUpgradeFuture},
        GdcfFuture,
    },
    upgrade::Upgrade,
    Gdcf,
};
use futures::{future::Either, task, Async, Future, Stream};
use gdcf_model::{song::NewgroundsSong, user::Creator};
use log::info;
use std::mem;
use crate::future::upgrade::UpgradeFuture;

#[allow(missing_debug_implementations)]
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

#[allow(missing_debug_implementations)]
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

impl<Req, A, C> ProcessRequestFuture<Req, A, C>
where
    A: ApiClient + MakeRequest<Req>,
    C: Cache + Store<Creator> + Store<NewgroundsSong> + CanCache<Req>,
    Req: Request,
{
    pub fn upgrade<Into>(self) -> UpgradeFuture<Self, A, C, Into, Req::Result>
    where
        Req::Result: Upgrade<C, Into>,
        A: MakeRequest<<Req::Result as Upgrade<C, Into>>::Request>,
        C: CanCache<<Req::Result as Upgrade<C, Into>>::Request>,
    {
        unimplemented!()
    }
}

impl<Req, A, C, T: Send + Sync + 'static> ProcessRequestFuture<Req, A, C>
where
    A: ApiClient + MakeRequest<Req>,
    C: Cache + Store<Creator> + Store<NewgroundsSong> + CanCache<Req>,
    Req: Request<Result = Vec<T>>,
{
    pub fn upgrade_all<Into>(self) -> MultiUpgradeFuture<Self, A, C, Into, T>
    where
        T: Upgrade<C, Into>,
        A: MakeRequest<<T as Upgrade<C, Into>>::Request>,
        C: CanCache<<T as Upgrade<C, Into>>::Request>,
    {
        unimplemented!()
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

impl<Req, A, C> GdcfFuture for ProcessRequestFuture<Req, A, C>
where
    A: ApiClient + MakeRequest<Req>,
    C: Cache + Store<Creator> + Store<NewgroundsSong> + CanCache<Req>,
    Req: Request,
{
    type ApiClient = A;
    type Cache = C;
    type Request = Req;
    type ToPeek = Req::Result;

    fn has_result_cached(&self) -> bool {
        match self.state {
            ProcessRequestFutureState::Outdated(..) | ProcessRequestFutureState::UpToDate(..) => true,
            _ => false,
        }
    }

    fn into_cached(self) -> Result<Result<Self::Item, Self>, Self::Error>
    where
        Self: Sized,
    {
        match self.state {
            ProcessRequestFutureState::Empty | ProcessRequestFutureState::Uncached(_) => Ok(Err(self)),
            ProcessRequestFutureState::Outdated(cache_entry, _) | ProcessRequestFutureState::UpToDate(cache_entry) => Ok(Ok(cache_entry)),
        }
    }

    fn new(gdcf: Gdcf<A, C>, request: &Self::Request) -> Result<Self, C::Err> {
        Ok(ProcessRequestFuture {
            forces_refresh: request.forces_refresh(),
            state: gdcf.process(request)?,
            gdcf,
        })
    }

    fn peek_cached<F: FnOnce(Self::ToPeek) -> Self::ToPeek>(self, f: F) -> Self {
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
}
