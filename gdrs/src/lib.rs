#![feature(pattern)]

extern crate gdcf;

#[macro_use]
extern crate serde_derive;
extern crate serde;
extern crate serde_urlencoded;
extern crate futures;
extern crate tokio_core;
extern crate hyper;

#[macro_use]
mod macros;
mod ser;
pub mod parse;

use std::str;

use gdcf::api::{GDClient, GDError};
use gdcf::api::request::level::LevelRequest;
use gdcf::api::request::level::LevelsRequest;

use futures::Future;
use futures::Stream;

use tokio_core::reactor::Handle;

use hyper::Client;
use hyper::client::HttpConnector;
use hyper::Method;
use hyper::Request;
use hyper::header::{ContentLength, ContentType};
use hyper::Error;
use hyper::StatusCode;

use ser::LevelRequestRem;
use ser::LevelsRequestRem;
use gdcf::model::RawObject;

#[derive(Serialize)]
#[serde(untagged)]
pub enum Req {
    #[serde(with = "LevelRequestRem")]
    DownloadLevel(LevelRequest),

    #[serde(with = "LevelsRequestRem")]
    GetLevels(LevelsRequest),
}

pub struct GDClientImpl {
    client: Client<HttpConnector>,
}

impl GDClientImpl {
    pub fn new(handle: &Handle) -> GDClientImpl {
        GDClientImpl {
            client: Client::new(handle),
        }
    }

    fn make_request(&self, endpoint: &str, req: Req) -> Request {
        let body = serde_urlencoded::to_string(req).unwrap();
        let mut req = Request::new(Method::Post, endpoint!(endpoint));

        println!("Making request {} to endpoint {}", body, endpoint);

        req.headers_mut().set(ContentType::form_url_encoded());
        req.headers_mut().set(ContentLength(body.len() as u64));
        req.set_body(body);

        req
    }
}

impl GDClient for GDClientImpl {
    fn handle(&self) -> &Handle {
        self.client.handle()
    }

    fn level(&self, req: LevelRequest) -> Box<Future<Item=RawObject, Error=GDError> + 'static> {
        let req = self.make_request("downloadGJLevel22", Req::DownloadLevel(req));

        prepare_future!(self.client.request(req), parse::level)
    }

    fn levels(&self, req: LevelsRequest) -> Box<Future<Item=Vec<RawObject>, Error=GDError>> {
        let req = self.make_request("getGJLevels21", Req::GetLevels(req));

        prepare_future!(self.client.request(req), parse::levels)
    }
}


fn convert_error(error: Error) -> GDError {
    match error {
        Error::Timeout => GDError::Timeout,
        Error::Utf8(err) => err.into(),
        _ => GDError::Unspecified
    }
}