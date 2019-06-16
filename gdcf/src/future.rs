use crate::{
    api::{request::PaginatableRequest, ApiClient},
    cache::{Cache, CacheEntry},
    error::{ApiError, GdcfError},
    ProcessRequest,
};
use futures::{future::Either, task, Async, Future, Stream};
use log::info;
use std::mem;

pub(crate) type GdcfInnerFuture<T, A, C> =
    Box<dyn Future<Item = CacheEntry<T, <C as Cache>::CacheEntryMeta>, Error = GdcfError<A, <C as Cache>::Err>> + Send + 'static>;

#[allow(missing_debug_implementations)]
pub enum GdcfFuture<T, A: ApiError, C: Cache> {
    Empty,
    Uncached(GdcfInnerFuture<T, A, C>),
    Outdated(CacheEntry<T, C::CacheEntryMeta>, GdcfInnerFuture<T, A, C>),
    UpToDate(CacheEntry<T, C::CacheEntryMeta>),
}

impl<T, A: ApiError, C: Cache> GdcfFuture<T, A, C> {
    pub fn cached(&self) -> Option<&CacheEntry<T, C::CacheEntryMeta>> {
        match self {
            GdcfFuture::Outdated(ref entry, _) | GdcfFuture::UpToDate(ref entry) => Some(entry),
            _ => None,
        }
    }

    pub fn inner_future(&self) -> Option<&GdcfInnerFuture<T, A, C>> {
        match self {
            GdcfFuture::Uncached(fut) | GdcfFuture::Outdated(_, fut) => Some(fut),
            _ => None,
        }
    }

    /// Constructs a future that resolves to an "extended" version of the object that would be
    /// returned by this future. For instance, if this future resolved to a `Level<u64, u64>`,
    /// the new one could resolve to a `Level<Option<NewgroundsSong>, u64>`
    ///
    /// ## Type parameters:
    ///
    /// + `T`: The type of the object the current future would resolve to. In the above example this
    /// would be `Level<u64, u64>`
    ///
    /// + `AddOn`: The type of the object we extend with. In the
    /// example above, this would be `Option<NewgroundsSong>`
    ///
    /// + `U`: The type of the object the
    /// new future will resolve to. In the above example this would be
    /// `Level<Option<NewgroundsSong>, u64>
    ///
    /// + `Look`: The type of the database-lookup closure.
    pub(crate) fn extend<AddOn, U, Look, Req, Comb, Fut>(
        self, lookup: Look, request: Req, combinator: Comb,
    ) -> Result<GdcfFuture<U, A, C>, C::Err>
    where
        T: Clone + Send + 'static,
        U: Send + 'static,
        AddOn: Clone + Send + 'static,
        Look: Clone + FnOnce(&T) -> Result<CacheEntry<AddOn, C::CacheEntryMeta>, C::Err> + Send + 'static,
        Req: Clone + FnOnce(&T) -> Fut + Send + 'static,
        Comb: Copy + Fn(T, Option<AddOn>) -> Option<U> + Send + Sync + 'static,
        Fut: Future<Item = CacheEntry<AddOn, C::CacheEntryMeta>, Error = GdcfError<A, C::Err>> + Send + 'static,
    {
        let future = match self {
            GdcfFuture::Empty => GdcfFuture::Empty,
            GdcfFuture::UpToDate(entry) =>
                match entry.extend(lookup, request, combinator)? {
                    (entry, None) => GdcfFuture::UpToDate(entry),
                    (cached, Some(future)) => GdcfFuture::Outdated(cached, Box::new(future)),
                },
            GdcfFuture::Uncached(future) => GdcfFuture::Uncached(Box::new(Self::extend_future(future, lookup, request, combinator))),
            GdcfFuture::Outdated(entry, future) =>
                if let (entry, None) = entry.extend(lookup.clone(), request.clone(), combinator)? {
                    GdcfFuture::Outdated(entry, Box::new(Self::extend_future(future, lookup, request, combinator)))
                } else {
                    GdcfFuture::Uncached(Box::new(Self::extend_future(future, lookup, request, combinator)))
                },
        };

        Ok(future)
    }

    fn extend_future<AddOn, U, Look, Req, Comb, Fut>(
        future: impl Future<Item = CacheEntry<T, C::CacheEntryMeta>, Error = GdcfError<A, C::Err>>, lookup: Look, request: Req,
        combinator: Comb,
    ) -> impl Future<Item = CacheEntry<U, C::CacheEntryMeta>, Error = GdcfError<A, C::Err>>
    where
        T: Clone + Send + 'static,
        U: Send + 'static,
        AddOn: Clone + Send + 'static,
        Look: FnOnce(&T) -> Result<CacheEntry<AddOn, C::CacheEntryMeta>, C::Err> + Send + 'static,
        Req: FnOnce(&T) -> Fut + Send + 'static,
        Comb: Copy + Fn(T, Option<AddOn>) -> Option<U> + Send + Sync + 'static,
        Fut: Future<Item = CacheEntry<AddOn, C::CacheEntryMeta>, Error = GdcfError<A, C::Err>> + Send + 'static,
    {
        future.and_then(move |cache_entry| {
            match cache_entry.extend(lookup, request, combinator) {
                Err(err) => Either::A(futures::future::err(GdcfError::Cache(err))),
                Ok((entry, None)) => Either::A(futures::future::ok(entry)),
                Ok((_, Some(future))) => Either::B(future),
            }
        })
    }
}

// FIXME: same thing as in cache.rs, specialization would be freaking awesome
impl<T, A: ApiError, C: Cache> GdcfFuture<Vec<T>, A, C> {
    pub(crate) fn extend_all<AddOn, U, Look, Req, Comb, Fut>(
        self, lookup: Look, request: Req, combinator: Comb,
    ) -> Result<GdcfFuture<Vec<U>, A, C>, C::Err>
    where
        T: Clone + Send + 'static,
        U: Send + 'static,
        AddOn: Clone + Send + 'static,
        Look: Clone + Fn(&T) -> Result<CacheEntry<AddOn, C::CacheEntryMeta>, C::Err> + Send + 'static,
        Req: Clone + Fn(&T) -> Fut + Send + 'static,
        Comb: Copy + Fn(T, Option<AddOn>) -> Option<U> + Send + Sync + 'static,
        Fut: Future<Item = CacheEntry<AddOn, C::CacheEntryMeta>, Error = GdcfError<A, C::Err>> + Send + 'static,
    {
        let future = match self {
            GdcfFuture::Empty => GdcfFuture::Empty,
            GdcfFuture::UpToDate(entry) =>
                match entry.extend_all(lookup, request, combinator)? {
                    (entry, None) => GdcfFuture::UpToDate(entry),
                    (cached, Some(future)) => GdcfFuture::Outdated(cached, Box::new(future)),
                },
            GdcfFuture::Uncached(future) => GdcfFuture::Uncached(Box::new(Self::extend_future_all(future, lookup, request, combinator))),
            GdcfFuture::Outdated(entry, future) =>
                if let (entry, None) = entry.extend_all(lookup.clone(), request.clone(), combinator)? {
                    GdcfFuture::Outdated(entry, Box::new(Self::extend_future_all(future, lookup, request, combinator)))
                } else {
                    GdcfFuture::Uncached(Box::new(Self::extend_future_all(future, lookup, request, combinator)))
                },
        };

        Ok(future)
    }

    fn extend_future_all<AddOn, U, Look, Req, Comb, Fut>(
        future: impl Future<Item = CacheEntry<Vec<T>, C::CacheEntryMeta>, Error = GdcfError<A, C::Err>>, lookup: Look, request: Req,
        combinator: Comb,
    ) -> impl Future<Item = CacheEntry<Vec<U>, C::CacheEntryMeta>, Error = GdcfError<A, C::Err>>
    where
        T: Clone + Send + 'static,
        U: Send + 'static,
        AddOn: Clone + Send + 'static,
        Look: Fn(&T) -> Result<CacheEntry<AddOn, C::CacheEntryMeta>, C::Err> + Send + 'static,
        Req: Fn(&T) -> Fut + Send + 'static,
        Comb: Copy + Fn(T, Option<AddOn>) -> Option<U> + Send + Sync + 'static,
        Fut: Future<Item = CacheEntry<AddOn, C::CacheEntryMeta>, Error = GdcfError<A, C::Err>> + Send + 'static,
    {
        future.and_then(move |cache_multi_entry| {
            match cache_multi_entry.extend_all(lookup, request, combinator) {
                Err(err) => Either::A(futures::future::err(GdcfError::Cache(err))),
                Ok((entry, None)) => Either::A(futures::future::ok(entry)),
                Ok((_, Some(future))) => Either::B(future),
            }
        })
    }
}

impl<T, A, C> Into<(CacheEntry<T, C::CacheEntryMeta>, Option<GdcfInnerFuture<T, A, C>>)> for GdcfFuture<T, A, C>
where
    A: ApiError,
    C: Cache,
{
    fn into(self) -> (CacheEntry<T, C::CacheEntryMeta>, Option<GdcfInnerFuture<T, A, C>>) {
        match self {
            GdcfFuture::Uncached(fut) => (CacheEntry::Missing, Some(fut)),
            GdcfFuture::Outdated(entry, fut) => (entry, Some(fut)),
            GdcfFuture::UpToDate(entry) => (entry, None),
            GdcfFuture::Empty => (CacheEntry::Missing, None),
        }
    }
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
    M: ProcessRequest<A, C, R, T>,
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
    M: ProcessRequest<A, C, R, T>,
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

                self.current_request = self.source.process_request(cur).map_err(GdcfError::Cache)?;

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
