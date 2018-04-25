use api::ApiClient;
use api::request::MakeRequest;
use futures::Future;
use api::response::ProcessedResponse;
use model::{RawObject, ObjectType, FromRawObject};
use cache::Cache;

pub(crate) trait ApiClientExt: ApiClient {
    fn make<R: MakeRequest + 'static>(&self, request: R) -> Box<Future<Item=ProcessedResponse, Error=()> + 'static> {
        Box::new(request.make(self)
            .map_err(move |err| error!("Unexpected error while processing api response to request {}: {:?}", request, err)))
    }
}

pub(crate) trait CacheExt: Cache {
    fn store_raw(&mut self, raw_object: &RawObject) {
        debug!("Received a {:?}, attempting to cache", raw_object.object_type);

        let err = match raw_object.object_type {
            ObjectType::Level => store!(self, store_level, raw_object),
            ObjectType::PartialLevel => store!(self, store_partial_level, raw_object),
            ObjectType::NewgroundsSong => store!(self, store_song, raw_object)
        };

        if let Err(err) = err {
            error!(
                "Unexpected error while constructing object {:?}: {:?}",
                raw_object.object_type, err
            )
        }
    }
}

impl<T> ApiClientExt for T
    where
        T: ApiClient
{}

impl<T> CacheExt for T
    where
        T: Cache
{}