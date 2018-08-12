use api::request::LevelsRequest;
use api::request::Request;
use api::response::ProcessedResponse;
use cache::Cache;
use error::CacheError;
//use ext::CacheExt;
use model::GDObject;

pub trait Cacher<R: Request> {
    fn store_response<C>(cache: &mut C, request: &R, response: ProcessedResponse) -> Result<(), CacheError<C::Err>>
        where
            C: Cache;
}

#[derive(Debug)]
pub struct DefaultCacher;

impl<R: Request> Cacher<R> for DefaultCacher {
    fn store_response<C>(cache: &mut C, request: &R, response: ProcessedResponse) -> Result<(), CacheError<C::Err>>
        where
            C: Cache
    {
        for object in response {
            cache.store_object(object)?
        }

        Ok(())
    }
}

#[derive(Debug)]
pub struct LevelsRequestCacher;

impl Cacher<LevelsRequest> for LevelsRequestCacher {
    fn store_response<C>(cache: &mut C, request: &LevelsRequest, response: ProcessedResponse) -> Result<(), CacheError<C::Err>>
        where
            C: Cache
    {
        let mut levels = Vec::new();

        for object in response {
            match object {
                GDObject::PartialLevel(lvl) => levels.push(lvl),
                other => cache.store_object(other)?
            }
        }

        cache.store_partial_levels(request, levels)
    }
}