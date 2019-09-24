use crate::{
    api::{client::MakeRequest, request::PaginatableRequest, ApiClient},
    cache::{Cache, CacheEntry, CanCache, Store},
    error::{ApiError, Error},
    future::{
        process::ProcessRequestFuture,
        upgrade::{MultiUpgradeFuture, UpgradeFuture},
        GdcfFuture,
    },
    upgrade::Upgradable,
    Gdcf,
};
use futures::{task, Async, Stream};
use gdcf_model::{song::NewgroundsSong, user::Creator};
use log::{debug, info, trace};

#[derive(Debug)]
pub struct GdcfStream<F: GdcfFuture>
where
    F::BaseRequest: PaginatableRequest,
{
    request: F::BaseRequest,
    current_future: F,
}

impl<F, T> GdcfStream<F>
where
    F::BaseRequest: PaginatableRequest,
    F: GdcfFuture<GdcfItem = Vec<T>>,
    T: Send + Sync + 'static,
{
    pub fn upgrade_all<Into>(self) -> GdcfStream<MultiUpgradeFuture<F, Into, T>>
    where
        T: Upgradable<F::Cache, Into>,
        F::ApiClient: MakeRequest<<T as Upgradable<F::Cache, Into>>::Request>,
        F::Cache: CanCache<<T as Upgradable<F::Cache, Into>>::Request>,
    {
        GdcfStream {
            current_future: MultiUpgradeFuture::upgrade_from(self.current_future),
            request: self.request,
        }
    }
}

impl<F: GdcfFuture> GdcfStream<F>
where
    F::BaseRequest: PaginatableRequest,
{
    pub fn upgrade<Into>(self) -> GdcfStream<UpgradeFuture<F, Into, F::GdcfItem>>
    where
        F::GdcfItem: Upgradable<F::Cache, Into>,
        F::ApiClient: MakeRequest<<F::GdcfItem as Upgradable<F::Cache, Into>>::Request>,
        F::Cache: CanCache<<F::GdcfItem as Upgradable<F::Cache, Into>>::Request>,
    {
        GdcfStream {
            current_future: UpgradeFuture::upgrade_from(self.current_future),
            request: self.request,
        }
    }
}

impl<A, C, Req> GdcfStream<ProcessRequestFuture<Req, A, C>>
where
    C: Store<NewgroundsSong> + Store<Creator> + Cache + CanCache<Req>,
    A: ApiClient + MakeRequest<Req>,
    Req: PaginatableRequest,
{
    pub(crate) fn new(gdcf: Gdcf<A, C>, request: Req) -> Result<Self, C::Err> {
        Ok(GdcfStream {
            current_future: ProcessRequestFuture::new(gdcf, &request)?,
            request,
        })
    }
}

impl<F> Stream for GdcfStream<F>
where
    F: GdcfFuture + std::fmt::Debug,
    F::BaseRequest: PaginatableRequest,
{
    type Error = Error<<F::ApiClient as ApiClient>::Err, <F::Cache as Cache>::Err>;
    type Item = CacheEntry<F::GdcfItem, <F::Cache as Cache>::CacheEntryMeta>;

    fn poll(&mut self) -> Result<Async<Option<Self::Item>>, Self::Error> {
        match self.current_future.poll() {
            Ok(Async::NotReady) => {
                trace!("Future {:?} not ready", self.current_future);

                Ok(Async::NotReady)
            },

            Ok(Async::Ready(page)) => {
                task::current().notify();

                info!("Advancing GdcfStream over {} by one page!", self.request);

                self.request.next();
                self.current_future = F::new(self.current_future.gdcf(), &self.request).map_err(Error::Cache)?;

                debug!("Request is now {}", self.request);
                trace!("New future is {:?}", self.current_future);

                Ok(Async::Ready(Some(page)))
            },

            Err(Error::Api(ref err)) if err.is_no_result() => {
                info!("Stream over request {} terminating due to exhaustion!", self.request);

                Ok(Async::Ready(None))
            },

            Err(err) => Err(err),
        }
    }
}
