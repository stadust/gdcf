#![feature(pattern)]
#![feature(try_from)]
#![deny(
    bare_trait_objects,
    missing_debug_implementations,
    unused_extern_crates,
    patterns_in_fns_without_body,
    stable_features,
    unknown_lints,
    unused_features,
    unused_imports,
    unused_parens
)]

extern crate futures;
extern crate gdcf;
extern crate hyper;
#[macro_use]
extern crate log;
extern crate serde;
#[macro_use]
extern crate serde_derive;
extern crate serde_urlencoded;

use futures::{future::Executor, Future, Stream};
use gdcf::{
    api::{
        client::ApiFuture,
        request::level::{LevelRequest, LevelsRequest},
        ApiClient,
    },
    error::ApiError,
};
use hyper::{
    client::{Builder, HttpConnector},
    header::HeaderValue,
    Body, Client, Error, Method, Request, StatusCode,
};
use ser::{LevelRequestRem, LevelsRequestRem};
use std::str;

#[macro_use]
mod macros;
pub mod parse;
mod ser;

#[derive(Serialize, Debug)]
#[serde(untagged)]
pub enum Req<'a> {
    #[serde(with = "LevelRequestRem")]
    DownloadLevel(&'a LevelRequest),

    #[serde(with = "LevelsRequestRem")]
    GetLevels(&'a LevelsRequest),
}

#[derive(Debug)]
pub struct BoomlingsClient {
    client: Client<HttpConnector>,
}

impl BoomlingsClient {
    pub fn new() -> BoomlingsClient {
        info!("Creating new BoomlingsApiClient");

        BoomlingsClient { client: Client::new() }
    }

    pub fn with_exec<E>(exec: E) -> Self
    where
        E: Executor<Box<dyn Future<Item = (), Error = ()> + Send>> + Send + Sync + 'static,
    {
        let client = Builder::default().executor(exec).build_http();

        BoomlingsClient { client }
    }

    fn make_request(&self, endpoint: &str, req: Req) -> Request<Body> {
        let body = serde_urlencoded::to_string(req).unwrap();
        let len = body.len();

        debug!("Preparing request {} to {}", body, endpoint);

        let mut req = Request::new(Body::from(body));

        *req.method_mut() = Method::POST;
        *req.uri_mut() = endpoint!(endpoint);
        req.headers_mut()
            .insert("Content-Type", HeaderValue::from_str("application/x-www-form-urlencoded").unwrap());
        req.headers_mut()
            .insert("Content-Length", HeaderValue::from_str(&len.to_string()).unwrap());

        req
    }
}

impl ApiClient for BoomlingsClient {
    type Err = Error;

    fn level(&self, req: &LevelRequest) -> ApiFuture<Self::Err> {
        let req = self.make_request("downloadGJLevel22", Req::DownloadLevel(req));

        prepare_future!(self.client.request(req), parse::level)
    }

    fn levels(&self, req: &LevelsRequest) -> ApiFuture<Self::Err> {
        let req = self.make_request("getGJLevels21", Req::GetLevels(req));

        prepare_future!(self.client.request(req), parse::levels)
    }
}
