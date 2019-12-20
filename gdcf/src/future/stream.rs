use crate::{
    api::{client::MakeRequest, request::PaginatableRequest, ApiClient},
    cache::{Cache, CacheEntry, CanCache, Lookup, Store},
    error::{ApiError, CacheError, Error},
    future::{process::ProcessRequestFuture, upgrade::UpgradeFuture, StreamableFuture},
    upgrade::Upgradable,
    Gdcf,
};
use futures::{task, Async, Stream};
use gdcf_model::{song::NewgroundsSong, user::Creator};
use log::{debug, info, trace};
use std::marker::PhantomData;

#[derive(Debug)]
pub struct GdcfStream<A: ApiClient, C: Cache, F: StreamableFuture<A, C>> {
    current_future: Option<F>,
    gdcf: Gdcf<A, C>,
}

impl<A: ApiClient, C: Cache, F: StreamableFuture<A, C>> GdcfStream<A, C, F> {
    pub fn new(gdcf: Gdcf<A, C>, future: F) -> Self {
        GdcfStream {
            current_future: Some(future),
            gdcf,
        }
    }
}

impl<A: ApiClient, C: Cache, F: StreamableFuture<A, C>> Stream for GdcfStream<A, C, F> {
    type Error = F::Error;
    type Item = F::Item;

    fn poll(&mut self) -> Result<Async<Option<Self::Item>>, Self::Error> {
        if let Some(ref mut current_future) = self.current_future {
            match current_future.poll() {
                Ok(Async::NotReady) => Ok(Async::NotReady),

                Ok(Async::Ready(page)) => {
                    // We cannot move out of borrowed context, which means we have to "trick" rust into allowing up to
                    // swap out the futures by using an Option
                    self.current_future = self
                        .current_future
                        .take()
                        .map(|current_future| current_future.next(&self.gdcf))
                        .transpose()?;

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
