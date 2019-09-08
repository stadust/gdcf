use crate::{
    api::{request::Request, ApiClient},
    cache::Cache,
    error::GdcfError,
    Gdcf,
};
use futures::Future;

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
pub trait GdcfFuture: Future {
    #[doc(hidden)]
    type ToPeek;

    type Request: Request;
    type Cache: Cache;
    type ApiClient: ApiClient;

    /// Returns whether the result of this future can be satisfied from cache
    fn has_result_cached(&self) -> bool;

    /// Gets the result of this future from the cache, if it is cached. If it isn't cached,
    /// `Ok(Err(self))` is returned. `Err(_)` is returned if a cache access fails.
    fn into_cached(self) -> Result<Result<Self::Item, Self>, Self::Error>
    where
        Self: Sized;

    fn new(gdcf: Gdcf<Self::ApiClient, Self::Cache>, request: &Self::Request) -> Result<Self, <Self::Cache as Cache>::Err>
    where
        Self: Sized;

    #[doc(hidden)]
    fn peek_cached<F: FnOnce(Self::ToPeek) -> Self::ToPeek>(self, f: F) -> Self;
}
