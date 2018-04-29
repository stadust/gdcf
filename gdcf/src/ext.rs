use api::ApiClient;
use api::response::ProcessedResponse;
use cache::Cache;
use error::CacheError;
use futures::Future;
use model::GDObject;
use api::request::Request;

pub(crate) trait ApiClientExt: ApiClient {
    fn make<R: Request + 'static>(&self, request: R) -> Box<Future<Item=ProcessedResponse, Error=()> + 'static> {
        Box::new(request.make(self)
            .map_err(move |err| error!("Unexpected error while processing api response to request {}: {:?}", request, err)))
    }
}

pub(crate) trait CacheExt: Cache {
    fn store_all(&mut self, objects: impl IntoIterator<Item=GDObject>) -> Result<(), Vec<CacheError<Self::Err>>> {
        let errors = objects.into_iter()
            .map(|object| self.store_object(object))
            .filter(|result| result.is_err())
            .map(|result| result.unwrap_err())
            .collect::<Vec<_>>();

        if errors.is_empty() {
            Ok(())
        } else {
            Err(errors)
        }
    }

    fn store_object(&mut self, object: GDObject) -> Result<(), CacheError<Self::Err>> {
        info!("Caching {}", object);

        match object {
            GDObject::Level(level) => self.store_level(level),
            GDObject::PartialLevel(level) => self.store_partial_level(level),
            GDObject::NewgroundsSong(song) => self.store_song(song)
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