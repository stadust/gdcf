//! Module containing GDCF's future types

use futures::Future;

use crate::{api::ApiClient, cache::Cache, error::Error, future::stream::GdcfStream};

pub mod process;
pub(crate) mod refresh;
pub mod stream;
pub mod upgrade;

pub trait PeekableFuture: Future + Sized {
    fn peek<F: FnOnce(Self::Item) -> Result<Self::Item, Self::Error>>(self, f: F) -> Result<Self, Self::Error>;
    //fn can_peek(&self) -> bool;
}

pub trait CloneablePeekFuture: PeekableFuture {
    fn clone_peek(&self) -> Result<Self::Item, ()>;
}

pub trait StreamableFuture<A: ApiClient, C: Cache>: Future<Error = Error<A::Err, C::Err>> + Sized {
    fn next(self) -> Result<Self, Self::Error>;

    fn stream(self) -> GdcfStream<A, C, Self> {
        GdcfStream::new(self)
    }
}
