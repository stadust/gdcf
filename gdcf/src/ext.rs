use api::ApiClient;
use api::request::Request;
use api::response::ProcessedResponse;
//use cache::Cache;
//use error::CacheError;
use futures::Future;
//use model::GDObject;
use std::fmt::Display;

pub trait ApiClientExt: ApiClient {
    fn make<R: Request + 'static>(&self, request: &R) -> Box<dyn Future<Item=ProcessedResponse, Error=()> + 'static> {
        let req_str = request.to_string();
        Box::new(request.make(self)
            .map_err(move |err| error!("Unexpected error while processing api response to request {}: {:?}", req_str, err)))
    }
}
/*
pub trait CacheExt: Cache {
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
            //GDObject::PartialLevel(level) => self.store_partial_level(level),
            GDObject::NewgroundsSong(song) => self.store_song(song),
            _ => panic!()
        }
    }
}*/

pub trait Join: Iterator {
    fn join(self, seperator: &str) -> String
        where
            Self::Item: Display,
            Self: Sized,
    {
        let mut result = String::new();
        let mut sep = "";

        for t in self {
            result = format!("{}{}{}", result, sep, t);
            sep = seperator;
        }

        result
    }

    fn join_quoted(self, seperator: &str) -> String
        where
            Self::Item: Display,
            Self: Sized,
    {
        let mut result = String::new();
        let mut sep = "";

        for t in self {
            result = format!("{}{}'{}'", result, sep, t);
            sep = seperator;
        }

        result
    }
}

impl<T> ApiClientExt for T
    where
        T: ApiClient
{}

/*impl<T> CacheExt for T
    where
        T: Cache
{}
*/
impl<I> Join for I
    where
        I: Iterator,
{}
