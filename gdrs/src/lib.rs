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
extern crate tokio_retry;

use futures::{future::Executor, Future, Stream};
use gdcf::{
    api::{
        client::ApiFuture,
        request::{
            level::{LevelRequest, LevelsRequest},
            user::UserRequest,
        },
        response::ProcessedResponse,
        ApiClient,
    },
    error::ApiError,
};
use hyper::{
    client::{Builder, HttpConnector},
    header::HeaderValue,
    Body, Client, Error, Method, Request, StatusCode,
};
use ser::{LevelRequestRem, LevelsRequestRem, UserRequestRem};
use std::str;
use tokio_retry::{
    strategy::{jitter, ExponentialBackoff},
    Action, Condition, Error as RetryError, RetryIf,
};

#[macro_use]
mod macros;
pub mod parse;
mod ser;

#[derive(Serialize, Debug)]
#[serde(untagged)]
pub enum Req {
    #[serde(with = "LevelRequestRem")]
    LevelRequest(LevelRequest),

    #[serde(with = "LevelsRequestRem")]
    LevelsRequest(LevelsRequest),

    #[serde(with = "UserRequestRem")]
    UserRequest(UserRequest),
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
}

impl ApiClient for BoomlingsClient {
    type Err = Error;

    api_call!(level, LevelRequest, "downloadGJLevel22", parse::level);

    api_call!(levels, LevelsRequest, "getGJLevels21", parse::levels);

    api_call!(user, UserRequest, "getGJUserInfo20", parse::user);
}

struct ApiRequestAction {
    client: Client<HttpConnector>,
    endpoint: &'static str,
    request: Req,
    parser: fn(&str) -> Result<ProcessedResponse, ApiError<Error>>,
}

struct ApiRetryCondition;

impl Condition<ApiError<Error>> for ApiRetryCondition {
    fn should_retry(&mut self, error: &ApiError<Error>) -> bool {
        match error {
            ApiError::Custom(_) => {
                warn!("Encountered retryable error: {:?}", error);
                true
            },
            _ => false,
        }
    }
}

impl Action for ApiRequestAction {
    type Error = ApiError<Error>;
    type Future = ApiFuture<Error>;
    type Item = ProcessedResponse;

    fn run(&mut self) -> ApiFuture<Error> {
        let req = make_request(self.endpoint, &self.request);

        let parser = self.parser.clone();
        let future = self
            .client
            .request(req)
            .map_err(|err| ApiError::Custom(err))
            .and_then(|resp| {
                debug!("Received {} response", resp.status());

                match resp.status() {
                    StatusCode::INTERNAL_SERVER_ERROR => Err(ApiError::InternalServerError),
                    StatusCode::NOT_FOUND => Err(ApiError::NoData),
                    _ => Ok(resp),
                }
            }).and_then(move |resp| {
                resp.into_body()
                    .concat2()
                    .map_err(|err| ApiError::Custom(err))
                    .and_then(move |body| {
                        match str::from_utf8(&body) {
                            Ok(body) => {
                                trace!("Received response {}", body);

                                parser(body)
                            },
                            Err(_) => Err(ApiError::UnexpectedFormat),
                        }
                    })
            });

        Box::new(future)
    }
}

fn make_request(endpoint: &str, req: &Req) -> Request<Body> {
    let body = serde_urlencoded::to_string(req).unwrap();
    let len = body.len();

    info!("Preparing request {} to {}", body, endpoint);

    let mut req = Request::new(Body::from(body));

    *req.method_mut() = Method::POST;
    *req.uri_mut() = endpoint!(endpoint);
    req.headers_mut()
        .insert("Content-Type", HeaderValue::from_str("application/x-www-form-urlencoded").unwrap());
    req.headers_mut()
        .insert("Content-Length", HeaderValue::from_str(&len.to_string()).unwrap());

    req
}
