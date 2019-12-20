use futures::{Async, Future};

use gdcf_model::{song::NewgroundsSong, user::Creator};

use crate::{
    api::{request::Request, ApiClient},
    cache::{Cache, CacheEntry, CreatorKey, NewgroundsSongKey, Store},
    error::Error,
    Gdcf,
};

pub mod process;
pub(crate) mod refresh;
pub mod stream;
pub mod upgrade;

pub trait PeekableFuture: Future + Sized {
    fn peek<F: FnOnce(Self::Item) -> Result<Self::Item, Self::Error>>(self, f: F) -> Result<Self, Self::Error>;
    //fn can_peek(&self) -> bool;
}

pub trait StreamableFuture<A: ApiClient, C: Cache>: Future<Error = Error<A::Err, C::Err>> + Sized {
    fn next(self, gdcf: &Gdcf<A, C>) -> Result<Self, Self::Error>;

    // this is probably a better solution
    /*fn stream(self) -> GdcfStream<A, C, Self> {
        GdcfStream::new(self.gdcf.clone(), self)
    }*/
}
/*
// FIXME: we probably do not want this to be its own trait. Right now it is so we can keep the Clone
// bounds contained
pub trait CloneCached: GdcfFuture {
    fn clone_cached(&self) -> Result<CacheEntry<Self::GdcfItem, <Self::Cache as Cache>::CacheEntryMeta>, ()>;
}
*/
