use crate::{
    api::{
        client::{MakeRequest, Response},
        request::Request,
        ApiClient,
    },
    cache::{Cache, CacheEntry, CanCache, Store},
    error::{ApiError, Error},
    Gdcf, Secondary,
};
use futures::{Async, Future};
use gdcf_model::{song::NewgroundsSong, user::Creator};
use log::warn;

pub(crate) struct RefreshCacheFuture<Req, A, C>
where
    Req: Request,
    A: ApiClient + MakeRequest<Req>,
    C: Cache + Store<Creator> + Store<NewgroundsSong> + CanCache<Req>,
{
    inner: <A as MakeRequest<Req>>::Future,
    gdcf: Gdcf<A, C>,
    cache_key: u64,
}

impl<Req, A, C> std::fmt::Debug for RefreshCacheFuture<Req, A, C>
where
    Req: Request,
    A: ApiClient + MakeRequest<Req>,
    C: Cache + Store<Creator> + Store<NewgroundsSong> + CanCache<Req>,
{
    fn fmt(&self, fmt: &mut std::fmt::Formatter) -> std::fmt::Result {
        fmt.debug_struct("RefreshCacheFuture").field("cache_key", &self.cache_key).finish()
    }
}

impl<Req, A, C> RefreshCacheFuture<Req, A, C>
where
    Req: Request,
    A: ApiClient + MakeRequest<Req>,
    C: Cache + Store<Creator> + Store<NewgroundsSong> + CanCache<Req>,
{
    pub(crate) fn new(gdcf: Gdcf<A, C>, cache_key: u64, inner: <A as MakeRequest<Req>>::Future) -> Self {
        RefreshCacheFuture { inner, gdcf, cache_key }
    }
}

impl<Req, A, C> Future for RefreshCacheFuture<Req, A, C>
where
    Req: Request,
    A: ApiClient + MakeRequest<Req>,
    C: Cache + Store<Creator> + Store<NewgroundsSong> + CanCache<Req>,
{
    type Error = Error<A::Err, C::Err>;
    type Item = CacheEntry<Req::Result, C::CacheEntryMeta>;

    fn poll(&mut self) -> Result<Async<Self::Item>, Self::Error> {
        let mut cache = self.gdcf.cache();

        match self.inner.poll() {
            Ok(Async::NotReady) => Ok(Async::NotReady),
            Err(ref api_error) if api_error.is_no_result() => {
                // TODO: maybe mark malformed data as absent as well

                warn!("Request yielded no result, marking as absent");

                Store::<Req::Result>::mark_absent(&mut cache, self.cache_key)
                    .map(|entry_info| Async::Ready(CacheEntry::MarkedAbsent(entry_info)))
                    .map_err(Error::Cache)
            },
            Err(api_error) => Err(Error::Api(api_error)),
            Ok(Async::Ready(response)) =>
                match response {
                    Response::Exact(what_we_want) =>
                        cache
                            .store(&what_we_want, self.cache_key)
                            .map(|entry_info| Async::Ready(CacheEntry::Cached(what_we_want, entry_info)))
                            .map_err(Error::Cache),
                    Response::More(what_we_want, excess) => {
                        for object in &excess {
                            match object {
                                Secondary::NewgroundsSong(song) => cache.store(song, song.song_id),
                                Secondary::Creator(creator) => cache.store(creator, creator.user_id),
                                Secondary::MissingCreator(cid) => Store::<Creator>::mark_absent(&mut cache, *cid),
                                Secondary::MissingNewgroundsSong(nid) => Store::<NewgroundsSong>::mark_absent(&mut cache, *nid),
                            }
                            .map_err(Error::Cache)?;
                        }

                        cache
                            .store(&what_we_want, self.cache_key)
                            .map(|entry_info| Async::Ready(CacheEntry::Cached(what_we_want, entry_info)))
                            .map_err(Error::Cache)
                    },
                },
        }
    }
}
