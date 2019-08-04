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

use crate::{
    error::ApiError,
    handle::Handler,
    ser::{LevelCommentsRequestRem, LevelRequestRem, LevelsRequestRem, ProfileCommentsRequestRem, UserRequestRem, UserSearchRequestRem},
};
use futures::{future::Executor, Future, Stream};
use gdcf::api::{
    client::{ApiFuture, MakeRequest, Response},
    request::{
        comment::{LevelCommentsRequest, ProfileCommentsRequest},
        level::{LevelRequest, LevelsRequest},
        user::{UserRequest, UserSearchRequest},
        Request as GdcfRequest,
    },
    ApiClient,
};
use hyper::{
    client::{Builder, HttpConnector},
    header::HeaderValue,
    Body, Client, Method, Request, StatusCode,
};
use log::{debug, error, info, trace, warn};
use serde_derive::Serialize;
use std::str;
use tokio_retry::{
    strategy::{jitter, ExponentialBackoff},
    Action, Condition, Error as RetryError, RetryIf,
};

#[macro_use]
mod macros;
pub mod error;
pub mod handle;
mod ser;

#[derive(Serialize, Debug)]
#[serde(untagged)]
pub enum Req<'a> {
    #[serde(with = "LevelRequestRem")]
    LevelRequest(&'a LevelRequest),

    #[serde(with = "LevelsRequestRem")]
    LevelsRequest(&'a LevelsRequest),

    #[serde(with = "UserRequestRem")]
    UserRequest(&'a UserRequest),

    #[serde(with = "UserSearchRequestRem")]
    UserSearchRequest(&'a UserSearchRequest),

    #[serde(with = "LevelCommentsRequestRem")]
    LevelCommentsRequest(&'a LevelCommentsRequest),

    #[serde(with = "ProfileCommentsRequestRem")]
    ProfileCommentsRequest(&'a ProfileCommentsRequest),
}

#[derive(Debug, Default, Clone)]
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
    type Err = ApiError;
}

impl<R: GdcfRequest + Handler> MakeRequest<R> for BoomlingsClient {
    fn make(&self, request: R) -> ApiFuture<R, ApiError> {
        let action = ApiRequestAction {
            client: self.client.clone(),
            request,
        };

        let retry = ExponentialBackoff::from_millis(10).map(jitter).take(5);

        Box::new(RetryIf::spawn(retry, action, ApiRetryCondition).map_err(|err| {
            match err {
                RetryError::OperationError(e) => e,
                _ => unimplemented!(),
            }
        }))
    }
}

struct ApiRequestAction<R: GdcfRequest + Handler> {
    client: Client<HttpConnector>,
    request: R,
}

struct ApiRetryCondition;

impl Condition<ApiError> for ApiRetryCondition {
    fn should_retry(&mut self, error: &ApiError) -> bool {
        match error {
            ApiError::Custom(_) => {
                warn!("Encountered retryable error: {:?}", error);
                true
            },
            _ => false,
        }
    }
}

impl<R: GdcfRequest + Handler> Action for ApiRequestAction<R> {
    type Error = ApiError;
    type Future = ApiFuture<R, ApiError>;
    type Item = Response<R::Result>;

    fn run(&mut self) -> ApiFuture<R, ApiError> {
        let future = self
            .client
            .request(make_request(&self.request))
            .map_err(ApiError::Custom)
            .and_then(|resp| {
                debug!("Received {} response", resp.status());

                match resp.status() {
                    StatusCode::INTERNAL_SERVER_ERROR => Err(ApiError::InternalServerError),
                    StatusCode::NOT_FOUND => Err(ApiError::NoData),
                    _ => Ok(resp),
                }
            })
            .and_then(move |resp| {
                resp.into_body()
                    .concat2()
                    .map_err(ApiError::Custom)
                    .and_then(move |body| {
                        match str::from_utf8(&body) {
                            Ok(body) => {
                                trace!("Received response {}", body);

                                R::handle(&body)
                            },
                            Err(_) => Err(ApiError::UnexpectedFormat),
                        }
                    })
                    .map_err(|err| {
                        error!("Error processing request: {}", err);
                        err
                    })
            });

        Box::new(future)
    }
}

fn make_request<R: GdcfRequest + Handler>(request: &R) -> Request<Body> {
    let body = serde_urlencoded::to_string(request.to_req()).unwrap();
    let len = body.len();

    info!("Preparing request {} to {}", body, R::endpoint());

    let mut req = Request::new(Body::from(body));

    *req.method_mut() = Method::POST;
    *req.uri_mut() = R::endpoint().parse().unwrap(); //endpoint!(endpoint);
    req.headers_mut()
        .insert("Content-Type", HeaderValue::from_str("application/x-www-form-urlencoded").unwrap());
    req.headers_mut()
        .insert("Content-Length", HeaderValue::from_str(&len.to_string()).unwrap());

    req
}
