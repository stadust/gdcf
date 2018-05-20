#![feature(pattern)]
#![feature(try_from)]

extern crate futures;
extern crate gdcf;
extern crate hyper;
#[macro_use]
extern crate log;
extern crate serde;
#[macro_use]
extern crate serde_derive;
extern crate serde_urlencoded;
extern crate tokio_core;

use futures::Future;
use futures::Stream;
use gdcf::api::ApiClient;
use gdcf::api::client::ApiFuture;
use gdcf::api::request::level::LevelRequest;
use gdcf::api::request::level::LevelsRequest;
use gdcf::api::response::ProcessedResponse;
use gdcf::error::ApiError;
use gdcf::model::raw::RawObject;
use hyper::Client;
use hyper::client::HttpConnector;
use hyper::Error;
use hyper::header::{ContentLength, ContentType};
use hyper::Method;
use hyper::Request;
use hyper::StatusCode;
use ser::LevelRequestRem;
use ser::LevelsRequestRem;
use std::str;
use tokio_core::reactor::Handle;

#[macro_use]
mod macros;
pub mod parse;
mod ser;

#[derive(Serialize)]
#[serde(untagged)]
pub enum Req<'a> {
    #[serde(with = "LevelRequestRem")]
    DownloadLevel(&'a LevelRequest),

    #[serde(with = "LevelsRequestRem")]
    GetLevels(&'a LevelsRequest),
}

pub struct BoomlingsClient {
    client: Client<HttpConnector>,
}

impl BoomlingsClient {
    pub fn new(handle: &Handle) -> BoomlingsClient {
        info!("Creating new BoomlingsApiClient");

        BoomlingsClient {
            client: Client::new(handle),
        }
    }

    fn make_request(&self, endpoint: &str, req: Req) -> Request {
        let body = serde_urlencoded::to_string(req).unwrap();
        let mut req = Request::new(Method::Post, endpoint!(endpoint));

        debug!("Preparing request {} to {}", body, endpoint);

        req.headers_mut().set(ContentType::form_url_encoded());
        req.headers_mut().set(ContentLength(body.len() as u64));
        req.set_body(body);

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

    fn spawn<F>(&self, f: F)
        where
            F: Future<Item=(), Error=()> + 'static
    {
        debug!("Spawning a future!");

        self.client.handle().spawn(f);
    }
}
