#![feature(pattern)]

extern crate gdcf;

extern crate futures;
extern crate tokio_core;
extern crate hyper;
extern crate serde_urlencoded;

#[macro_use]
mod macros;
pub mod parse;

use std::str;

use gdcf::api::{GDClient, GDError};
use gdcf::model::GDObject;
use gdcf::api::request::level::LevelRequest;

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

pub struct GDClientImpl {
    client: Client<HttpConnector>,
}

impl GDClientImpl {
    pub fn new(handle: &Handle) -> GDClientImpl {
        GDClientImpl {
            client: Client::new(handle),
        }
    }
}

impl GDClient for GDClientImpl {
    fn handle(&self) -> &Handle {
        self.client.handle()
    }

    fn level(&self, req: LevelRequest) -> Box<Future<Item=GDObject, Error=GDError> + 'static> {
        let req = prepare_request!("downloadGJLevel22", req);

        prepare_future!(self.client.request(req), parse::level)
    }
}


fn convert_error(error: Error) -> GDError {
    match error {
        Error::Timeout => GDError::Timeout,
        Error::Utf8(err) => err.into(),
        _ => GDError::Unspecified
    }
}