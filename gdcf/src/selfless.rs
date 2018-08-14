use api::ApiClient;
use api::request::LevelsRequest;
use cache::Cache;
use model::PartialLevel;
use std::sync::Arc;
use std::sync::Mutex;
use error::CacheError;
use cache::Cache;
use api::request::LevelRequest;
use model::Level;
use api::response::ProcessedResponse;
use model::GDObject;
use futures::Future;

pub(crate) fn levels<A, C>(client: Arc<Mutex<A>>, cache: Arc<Mutex<C>>, req: LevelsRequest) -> GdcfFuture<Vec<PartialLevel>, C::Err, A::Err>
    where
        A: ApiClient,
        C: Cache
{
    let cache_clone = cache.clone();

    let process_response = move |response: ProcessedResponse| {
        let mut levels = Vec::new();
        let cache = lock!(cache_clone);

        for obj in response {
            match obj {
                GDObject::PartialLevel(level) => levels.push(level),
                _ => cache.store_object(&obj)?
            }
        }

        cache.store_partial_levels(&req, &levels);
        Ok(levels)
    };

    let cache = lock!(cache);
    let client = lock!(client);

    match cache.lookup_partial_levels(&req) {
        Ok(cached) => {
            if cache.is_expired(&cached) {
                GdcfFuture::outdated(cached.extract(), client.levels(&req).and_then(process_response))
            } else {
                GdcfFuture::up_to_date(cached.extract())
            }
        }

        Err(CacheError::CacheMiss) => GdcfFuture::absent(client.levels(&req).and_then(process_response)),

        Err(err) => panic!("Error accessing cache! {:?}", err)
    }
}

pub(crate) fn level<A, C>(client: Arc<Mutex<A>>, cache: Arc<Mutex<C>>, req: LevelRequest) -> GdcfFuture<Level, C::Err, A::Err>
    where
        A: ApiClient,
        C: Cache
{
    let cache_clone = cache.clone();
    let client_clone = client.clone();

    let process_response = move |response: ProcessedResponse| {

    };
}