use crate::{
    api::{client::MakeRequest, request::PaginatableRequest, ApiClient},
    cache::{Cache, CanCache, Store},
    error::{ApiError, GdcfError},
    future::{process::ProcessRequestFuture, GdcfFuture},
    Gdcf,
};
use futures::{task, Async, Stream};
use gdcf_model::{song::NewgroundsSong, user::Creator};
use log::info;

#[allow(missing_debug_implementations)]
pub struct GdcfStream<A: ApiClient, C: Cache, Req: PaginatableRequest, F: GdcfFuture> {
    gdcf: Gdcf<A, C>,
    request: Req,
    current_future: F,
}

impl<A, C, Req> GdcfStream<A, C, Req, ProcessRequestFuture<Req, A, C>>
where
    C: Store<NewgroundsSong> + Store<Creator>,
    A: ApiClient + MakeRequest<Req>,
    C: Cache + CanCache<Req>,
    Req: PaginatableRequest,
{
    pub(crate) fn new(gdcf: Gdcf<A, C>, request: Req) -> Result<Self, C::Err> {
        Ok(GdcfStream {
            current_future: gdcf.process(&request)?,
            request,
            gdcf,
        })
    }
}

impl<A, C, Req, F> Stream for GdcfStream<A, C, Req, F>
where
    A: ApiClient,
    C: Cache,
    Req: PaginatableRequest,
    F: GdcfFuture<Error = GdcfError<A::Err, C::Err>, Cache = C, ApiClient = A, Request = Req>,
{
    type Error = F::Error;
    type Item = F::Item;

    fn poll(&mut self) -> Result<Async<Option<Self::Item>>, Self::Error> {
        match self.current_future.poll() {
            Ok(Async::NotReady) => Ok(Async::NotReady),

            Ok(Async::Ready(page)) => {
                task::current().notify();

                self.request.next();
                self.current_future = F::new(self.gdcf.clone(), &self.request).map_err(GdcfError::Cache)?;

                Ok(Async::Ready(Some(page)))
            },

            Err(GdcfError::Api(ref err)) if err.is_no_result() => {
                info!("Stream over request {} terminating due to exhaustion!", self.request);

                Ok(Async::Ready(None))
            },

            Err(err) => Err(err),
        }
    }
}
