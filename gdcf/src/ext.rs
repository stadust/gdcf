use api::ApiClient;
use api::request::Request;
use api::response::ProcessedResponse;
use futures::Future;
use std::fmt::Display;

pub trait ApiClientExt: ApiClient {
    fn make<R: Request + 'static>(&self, request: &R) -> Box<dyn Future<Item=ProcessedResponse, Error=()> + 'static> {
        let req_str = request.to_string();
        Box::new(request.make(self)
            .map_err(move |err| error!("Unexpected error while processing api response to request {}: {:?}", req_str, err)))
    }
}

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

impl<I> Join for I
    where
        I: Iterator,
{}
