use crate::{
    api::{request::Request, ApiClient},
    cache::{Cache, CacheEntry, Store},
    error::GdcfError,
    Gdcf,
};
use futures::Async;
use gdcf_model::{song::NewgroundsSong, user::Creator};

pub mod process;
pub mod refresh;
pub mod stream;
pub mod upgrade;

/// Trait implemented by all futures from gdcf.
///
/// A [`GdcfFuture`] always represents an operation that might be completed immediately based on
/// cached data. In this case, it can be converted into the cached data directly without having to
/// ever poll the future. If the result from the cache isn't desired, or the result wasn't cached,
/// it can be spawned instead, potentially performing a request and updating the cache.
pub trait GdcfFuture /* : Future */ {
    type GdcfItem;

    type BaseRequest: Request;
    type Cache: Cache + Store<Creator> + Store<NewgroundsSong>;
    type ApiClient: ApiClient;

    /// Returns whether the result of this future can be satisfied from cache
    fn has_result_cached(&self) -> bool;

    /// Gets the result of this future from the cache, if it is cached. If it isn't cached,
    /// `Ok(Err(self))` is returned. `Err(_)` is returned if a cache access fails.
    fn into_cached(
        self,
    ) -> Result<
        Result<CacheEntry<Self::GdcfItem, <Self::Cache as Cache>::CacheEntryMeta>, Self>,
        GdcfError<<Self::ApiClient as ApiClient>::Err, <Self::Cache as Cache>::Err>,
    >
    where
        Self: Sized;

    fn new(gdcf: Gdcf<Self::ApiClient, Self::Cache>, request: &Self::BaseRequest) -> Result<Self, <Self::Cache as Cache>::Err>
    where
        Self: Sized;

    fn gdcf(&self) -> Gdcf<Self::ApiClient, Self::Cache>;
    fn forcing_refreshs(&self) -> bool;

    fn poll(
        &mut self,
    ) -> Result<
        Async<CacheEntry<Self::GdcfItem, <Self::Cache as Cache>::CacheEntryMeta>>,
        GdcfError<<Self::ApiClient as ApiClient>::Err, <Self::Cache as Cache>::Err>,
    >;

    #[doc(hidden)]
    fn peek_cached<F: FnOnce(Self::GdcfItem) -> Self::GdcfItem>(self, f: F) -> Self;
}
