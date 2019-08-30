use crate::{
    api::{
        client::MakeRequest,
        request::{PaginatableRequest, Request},
        ApiClient,
    },
    cache::{Cache, CacheEntry, CanCache, Store},
    error::{ApiError, GdcfError},
    future::{refresh::RefreshCacheFuture, GdcfFuture},
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
    type ToPeek = Req::Result;

    fn has_result_cached(&self) -> bool {
        match self {
            ProcessRequestFuture::Outdated(..) | ProcessRequestFuture::UpToDate(..) => true,
            _ => false,
        }
    }

    /*fn into_cached(self) -> Option<Self::Item> {
        match self {
            ProcessRequestFuture::Empty | ProcessRequestFuture::Uncached(_) => None,
            ProcessRequestFuture::Outdated(cached, _) | ProcessRequestFuture::UpToDate(cached) => Some(cached),
        }
    }*/

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
