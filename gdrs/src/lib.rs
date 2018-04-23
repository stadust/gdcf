#![feature(pattern)]

extern crate gdcf;

#[macro_use]
extern crate serde_derive;
extern crate futures;
extern crate hyper;
extern crate serde;
extern crate serde_urlencoded;
extern crate tokio_core;

#[macro_use]
mod macros;
pub mod parse;
mod ser;

use std::str;

use gdcf::api::request::level::LevelRequest;
use gdcf::api::request::level::LevelsRequest;
use gdcf::api::{ApiClient, GDError};

use futures::Future;
use futures::Stream;

use tokio_core::reactor::Handle;

use hyper::client::HttpConnector;
use hyper::header::{ContentLength, ContentType};
use hyper::Client;
use hyper::Error;
use hyper::Method;
use hyper::Request;
use hyper::StatusCode;

use gdcf::model::RawObject;
use ser::LevelRequestRem;
use ser::LevelsRequestRem;

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
        BoomlingsClient {
            client: Client::new(handle),
        }
    }

    fn make_request(&self, endpoint: &str, req: Req) -> Request {
        let body = serde_urlencoded::to_string(req).unwrap();
        let mut req = Request::new(Method::Post, endpoint!(endpoint));

        req.headers_mut().set(ContentType::form_url_encoded());
        req.headers_mut().set(ContentLength(body.len() as u64));
        req.set_body(body);

        req
    }
}

impl ApiClient for BoomlingsClient {
    fn level(&self, req: &LevelRequest) -> Box<Future<Item=RawObject, Error=GDError> + 'static> {
        let req = self.make_request("downloadGJLevel22", Req::DownloadLevel(req));

        prepare_future!(self.client.request(req), parse::level)
    }

    fn levels(&self, req: &LevelsRequest) -> Box<Future<Item=Vec<RawObject>, Error=GDError>> {
        let req = self.make_request("getGJLevels21", Req::GetLevels(req));

        prepare_future!(self.client.request(req), parse::levels)
    }

    fn spawn<F>(&self, f: F)
        where
            F: Future<Item=(), Error=()> + 'static
    {
        self.client.handle().spawn(f);
    }
}

fn convert_error(error: Error) -> GDError {
    match error {
        Error::Timeout => GDError::Timeout,
        Error::Utf8(err) => err.into(),
        _ => GDError::Unspecified,
    }
}
