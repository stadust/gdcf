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
        upgrade::{MultiUpgradeFuture, UpgradeFuture},
        GdcfFuture,
    },
    upgrade::Upgrade,
    Gdcf,
};
use futures::{future::Either, task, Async, Future, Stream};
use gdcf_model::{song::NewgroundsSong, user::Creator};
use log::info;
use std::mem;

#[allow(missing_debug_implementations)]
pub enum ProcessRequestFuture<Req, A, C>
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
        match self {
            ProcessRequestFuture::Empty => panic!("Future already polled to completion"),
            ProcessRequestFuture::Uncached(future) => future.poll(),
            ProcessRequestFuture::Outdated(_, future) => future.poll(),
            fut =>
                match std::mem::replace(fut, ProcessRequestFuture::Empty) {
                    ProcessRequestFuture::UpToDate(inner) => Ok(Async::Ready(inner)),
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
        match self {
            ProcessRequestFuture::Outdated(..) | ProcessRequestFuture::UpToDate(..) => true,
            _ => false,
        }
    }

    fn into_cached(self) -> Result<Result<Self::Item, Self>, Self::Error>
    where
        Self: Sized,
    {
        match self {
            ProcessRequestFuture::Empty | ProcessRequestFuture::Uncached(_) => Ok(Err(self)),
            ProcessRequestFuture::Outdated(cache_entry, _) | ProcessRequestFuture::UpToDate(cache_entry) => Ok(Ok(cache_entry)),
        }
    }

    fn new(gdcf: Gdcf<A, C>, request: &Self::Request) -> Result<Self, C::Err> {
        gdcf.process(request) // FIXME: error handling
    }

    fn peek_cached<F: FnOnce(Self::ToPeek) -> Self::ToPeek>(self, f: F) -> Self {
        match self {
            ProcessRequestFuture::Outdated(CacheEntry::Cached(object, meta), future) =>
                ProcessRequestFuture::Outdated(CacheEntry::Cached(f(object), meta), future),
            ProcessRequestFuture::UpToDate(CacheEntry::Cached(object, meta)) =>
                ProcessRequestFuture::UpToDate(CacheEntry::Cached(f(object), meta)),
            _ => self,
        }
    }
}
