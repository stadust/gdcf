use crate::{
    api::ApiClient,
    cache::Cache,
    error::{ApiError, Error},
    future::StreamableFuture,
};
use futures::{Async, Stream};
use std::marker::PhantomData;

#[derive(Debug)]
pub struct GdcfStream<A: ApiClient, C: Cache, F: StreamableFuture<A, C>> {
    current_future: Option<F>,
    _phantom: PhantomData<(A, C)>,
}

impl<A: ApiClient, C: Cache, F: StreamableFuture<A, C>> GdcfStream<A, C, F> {
    pub(crate) fn new(future: F) -> Self {
        GdcfStream {
            current_future: Some(future),
            _phantom: PhantomData,
        }
    }
}

// FIXME: figure out a way to terminate these streams if our Item is a collection type (like Vec<T>)
// and we receive an empty collection
impl<A: ApiClient, C: Cache, F: StreamableFuture<A, C>> Stream for GdcfStream<A, C, F> {
    type Error = F::Error;
    type Item = F::Item;

    fn poll(&mut self) -> Result<Async<Option<Self::Item>>, Self::Error> {
        if let Some(ref mut current_future) = self.current_future {
            match current_future.poll() {
                Ok(Async::NotReady) => Ok(Async::NotReady),

                Ok(Async::Ready(page)) => {
                    // We cannot move out of borrowed context, which means we have to "trick" rust into allowing us to
                    // swap out the futures by using an Option
                    self.current_future = self.current_future.take().map(|current_future| current_future.next()).transpose()?;

                    Ok(Async::Ready(Some(page)))
                },

                Err(Error::Api(ref err)) if err.is_no_result() => {
                    //info!("Stream over request {} terminating due to exhaustion!", self.request);

                    Ok(Async::Ready(None))
                },

                Err(err) => Err(err),
            }
        } else {
            Ok(Async::Ready(None))
        }
    }
}
