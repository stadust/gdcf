use futures::Async;

use gdcf_model::{song::NewgroundsSong, user::Creator};

use crate::{
    api::{request::Request, ApiClient},
    cache::{Cache, CacheEntry, Store},
    error::Error,
    Gdcf,
};

pub mod process;
pub(crate) mod refresh;
pub mod stream;
pub mod upgrade;

/// Trait implemented by all futures from gdcf.
///
/// A [`GdcfFuture`] always represents an operation that might be completed immediately based on
/// cached data. In this case, it can be converted into the cached data directly without having to
/// ever poll the future. If the result from the cache isn't desired, or the result wasn't cached,
/// it can be spawned instead, potentially performing a request and updating the cache.
pub trait GdcfFuture {
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
        Error<<Self::ApiClient as ApiClient>::Err, <Self::Cache as Cache>::Err>,
    >
    where
        Self: Sized;

    fn new(gdcf: Gdcf<Self::ApiClient, Self::Cache>, request: &Self::BaseRequest) -> Result<Self, <Self::Cache as Cache>::Err>
    where
        Self: Sized;

    fn gdcf(&self) -> Gdcf<Self::ApiClient, Self::Cache>;
    fn forcing_refreshes(&self) -> bool;

    /// Checks if the base object for this request is marked as absent in the cache or has been
    /// inferred to be absent
    ///
    /// Since all [`GdcfFuture`] are constructed in a decorator like way, this checks if the leaf
    /// future indicated that its result was absent.
    ///
    /// If the object is not cached, this method returns `false`! Absent means that the object was
    /// missing server sided, is not implied by a client side cache miss
    fn is_absent(&self) -> bool;

    fn poll(
        &mut self,
    ) -> Result<
        Async<CacheEntry<Self::GdcfItem, <Self::Cache as Cache>::CacheEntryMeta>>,
        Error<<Self::ApiClient as ApiClient>::Err, <Self::Cache as Cache>::Err>,
    >;

    #[doc(hidden)]
    fn peek_cached<F: FnOnce(Self::GdcfItem) -> Self::GdcfItem>(self, f: F) -> Self;
}

// FIXME: we probably do not want this to be its own trait. Right now it is so we can keep the Clone
// bounds contained
pub trait CloneCached: GdcfFuture {
    fn clone_cached(&self) -> Result<CacheEntry<Self::GdcfItem, <Self::Cache as Cache>::CacheEntryMeta>, ()>;
}
