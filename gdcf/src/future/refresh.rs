use crate::{
    api::{
        client::{MakeRequest, Response},
        request::Request,
        ApiClient,
    },
    cache::{Cache, CacheEntry, CanCache, CreatorKey, NewgroundsSongKey, Store},
    error::{ApiError, Error},
    Gdcf, Secondary,
};
use futures::{Async, Future};
use log::{info, warn};

pub(crate) struct RefreshCacheFuture<Req, A, C>
where
    Req: Request,
    A: ApiClient + MakeRequest<Req>,
    C: Cache + Store<CreatorKey> + Store<NewgroundsSongKey> + CanCache<Req>,
{
    inner: <A as MakeRequest<Req>>::Future,
    cache: C,
    pub(super) request: Req,
}

impl<Req, A, C> std::fmt::Debug for RefreshCacheFuture<Req, A, C>
where
    Req: Request + std::fmt::Debug,
    A: ApiClient + MakeRequest<Req>,
    C: Cache + Store<CreatorKey> + Store<NewgroundsSongKey> + CanCache<Req>,
{
    fn fmt(&self, fmt: &mut std::fmt::Formatter) -> std::fmt::Result {
        fmt.debug_struct("RefreshCacheFuture").field("request", &self.request).finish()
    }
}

impl<Req, A, C> RefreshCacheFuture<Req, A, C>
where
    Req: Request,
    A: ApiClient + MakeRequest<Req>,
    C: Cache + Store<CreatorKey> + Store<NewgroundsSongKey> + CanCache<Req>,
{
    pub(crate) fn new(gdcf: &Gdcf<A, C>, request: Req) -> Self {
        info!("Performing refresh on request {:?}", request);

        RefreshCacheFuture {
            inner: gdcf.client().make(&request),
            cache: gdcf.cache(),
            request,
        }
    }
}

impl<Req, A, C> Future for RefreshCacheFuture<Req, A, C>
where
    Req: Request,
    A: ApiClient + MakeRequest<Req>,
    C: Cache + Store<CreatorKey> + Store<NewgroundsSongKey> + CanCache<Req>,
{
    type Error = Error<A::Err, C::Err>;
    type Item = CacheEntry<Req::Result, C::CacheEntryMeta>;

    fn poll(&mut self) -> Result<Async<Self::Item>, Self::Error> {
        match self.inner.poll() {
            Ok(Async::NotReady) => Ok(Async::NotReady),
            Err(ref api_error) if api_error.is_no_result() => {
                // TODO: maybe mark malformed data as absent as well

                warn!("Request yielded no result, marking as absent");

                Store::<Req>::mark_absent(&mut self.cache, &self.request)
                    .map(|entry_info| Async::Ready(CacheEntry::MarkedAbsent(entry_info)))
                    .map_err(Error::Cache)
            },
            Err(api_error) => Err(Error::Api(api_error)),
            Ok(Async::Ready(response)) =>
                match response {
                    Response::Exact(what_we_want) =>
                        self.cache
                            .store(&what_we_want, &self.request)
                            .map(|entry_info| Async::Ready(CacheEntry::Cached(what_we_want, entry_info)))
                            .map_err(Error::Cache),
                    Response::More(what_we_want, excess) => {
                        for object in &excess {
                            match object {
                                Secondary::NewgroundsSong(song) => self.cache.store(song, &NewgroundsSongKey(song.song_id)),
                                Secondary::Creator(creator) => self.cache.store(creator, &CreatorKey(creator.user_id)),
                                Secondary::MissingCreator(cid) => Store::<CreatorKey>::mark_absent(&mut self.cache, &CreatorKey(*cid)),
                                Secondary::MissingNewgroundsSong(nid) =>
                                    Store::<NewgroundsSongKey>::mark_absent(&mut self.cache, &NewgroundsSongKey(*nid)),
                            }
                            .map_err(Error::Cache)?;
                        }

                        self.cache
                            .store(&what_we_want, &self.request)
                            .map(|entry_info| Async::Ready(CacheEntry::Cached(what_we_want, entry_info)))
                            .map_err(Error::Cache)
                    },
                },
        }
    }
}
