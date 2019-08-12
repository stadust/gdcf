use crate::{
    api::{
        client::MakeRequest,
        request::{PaginatableRequest, Request},
        ApiClient,
    },
    cache::{Cache, CacheEntry, CanCache, Store},
    error::{ApiError, GdcfError},
    future::refresh::RefreshCacheFuture,
    ProcessRequestOld,
};
use futures::{future::Either, task, Async, Future, Stream};
use gdcf_model::{song::NewgroundsSong, user::Creator};
use log::info;
use std::mem;

pub mod extend;
pub mod refresh;
pub mod stream;

pub trait GdcfFut: Future {
    fn has_result_cached(&self) -> bool;
    fn into_cached(self) -> Option<Self::Item>;
}

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

pub(crate) type GdcfInnerFuture<T, A, C> =
    Box<dyn Future<Item = CacheEntry<T, <C as Cache>::CacheEntryMeta>, Error = GdcfError<A, <C as Cache>::Err>> + Send + 'static>;

#[allow(missing_debug_implementations)]
pub enum GdcfFuture<T, A: ApiError, C: Cache> {
    Empty,
    Uncached(GdcfInnerFuture<T, A, C>),
    Outdated(CacheEntry<T, C::CacheEntryMeta>, GdcfInnerFuture<T, A, C>),
    UpToDate(CacheEntry<T, C::CacheEntryMeta>),
}

impl<T, A: ApiError, C: Cache> Future for GdcfFuture<T, A, C> {
    type Error = GdcfError<A, C::Err>;
    type Item = CacheEntry<T, C::CacheEntryMeta>;

    fn poll(&mut self) -> Result<Async<Self::Item>, Self::Error> {
        match self {
            GdcfFuture::Empty => Ok(Async::NotReady),
            GdcfFuture::Uncached(future) => future.poll(),
            GdcfFuture::Outdated(_, future) => future.poll(),
            fut =>
                match std::mem::replace(fut, GdcfFuture::Empty) {
                    GdcfFuture::UpToDate(inner) => Ok(Async::Ready(inner)),
                    _ => unreachable!(),
                },
        }
    }
}

#[allow(missing_debug_implementations)]
pub struct GdcfStream<A, C, R, T, M>
where
    R: PaginatableRequest,
    M: ProcessRequestOld<A, C, R, T>,
    A: ApiClient,
    C: Cache,
{
    pub(crate) next_request: R,
    pub(crate) current_request: GdcfFuture<T, A::Err, C>,
    pub(crate) source: M,
}

impl<A, C, R, T, M> Stream for GdcfStream<A, C, R, T, M>
where
    R: PaginatableRequest,
    M: ProcessRequestOld<A, C, R, T>,
    A: ApiClient,
    C: Cache,
{
    type Error = GdcfError<A::Err, C::Err>;
    type Item = CacheEntry<T, C::CacheEntryMeta>;

    fn poll(&mut self) -> Result<Async<Option<Self::Item>>, Self::Error> {
        match self.current_request.poll() {
            Ok(Async::NotReady) => Ok(Async::NotReady),

            Ok(Async::Ready(result)) => {
                task::current().notify();

                let next = self.next_request.next();
                let cur = mem::replace(&mut self.next_request, next);

                self.current_request = self.source.process_request_old(cur).map_err(GdcfError::Cache)?;

                Ok(Async::Ready(Some(result)))
            },

            Err(GdcfError::Api(ref err)) if err.is_no_result() => {
                info!("Stream over request {} terminating due to exhaustion!", self.next_request);

                Ok(Async::Ready(None))
            },

            Err(err) => Err(err),
        }
    }
}
