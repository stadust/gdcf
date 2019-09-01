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
use failure::_core::marker::PhantomData;
use futures::{
    future::{Executor, FromErr},
    stream::Concat2,
    Async, Future, Stream,
};
use gdcf::api::{
    client::{MakeRequest, Response},
    request::{
        comment::{LevelCommentsRequest, ProfileCommentsRequest},
        level::{LevelRequest, LevelsRequest},
        user::{UserRequest, UserSearchRequest},
        Request as GdcfRequest,
    },
    ApiClient,
};
use hyper::{
    client::{Builder, HttpConnector, ResponseFuture},
    header::HeaderValue,
    Body, Client, Method, Request, StatusCode,
};
use log::{debug, error, info, trace, warn};
use serde_derive::Serialize;
use std::{iter::Take, mem, str};
use tokio_retry::{strategy::ExponentialBackoff, Action, Condition, RetryIf};

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

#[allow(missing_debug_implementations)]
pub struct GdrsFuture<R: Handler> {
    inner: FromErr<RetryIf<Take<ExponentialBackoff>, ApiRequestAction<R>, ApiRetryCondition>, ApiError>,
}

impl<R: Handler> Future for GdrsFuture<R> {
    type Error = ApiError;
    type Item = Response<R::Result>;

    fn poll(&mut self) -> Result<Async<Self::Item>, Self::Error> {
        self.inner.poll()
    }
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

impl<R: Handler> MakeRequest<R> for BoomlingsClient {
    type Future = GdrsFuture<R>;

    fn make(&self, request: &R) -> GdrsFuture<R> {
        GdrsFuture {
            inner: RetryIf::spawn(
                ExponentialBackoff::from_millis(10).take(5),
                ApiRequestAction {
                    client: self.client.clone(),
                    encoded_request: serde_urlencoded::to_string(request.to_req()).unwrap(),
                    phantom: PhantomData,
                },
                ApiRetryCondition,
            )
            .from_err(),
        }
    }
}

struct ApiRequestAction<R: Handler> {
    client: Client<HttpConnector>,
    encoded_request: String,
    phantom: PhantomData<R>,
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

enum ProcessRequestFuture<R: Handler> {
    WaitingForResponse(ResponseFuture, PhantomData<R>),
    ProcessingResponse(Concat2<Body>),
}

impl<R: Handler> Future for ProcessRequestFuture<R> {
    type Error = ApiError;
    type Item = Response<R::Result>;

    fn poll(&mut self) -> Result<Async<Self::Item>, Self::Error> {
        let response_poll_result = match self {
            ProcessRequestFuture::WaitingForResponse(response_future, _) =>
                match response_future.poll() {
                    Ok(Async::NotReady) => return Ok(Async::NotReady),
                    Err(err) => {
                        error!("Error making request: {:?}", err);

                        return Err(ApiError::Custom(err))
                    },
                    Ok(Async::Ready(response)) => {
                        debug!("Received {} response", response.status());

                        match response.status() {
                            StatusCode::INTERNAL_SERVER_ERROR => return Err(ApiError::InternalServerError),
                            StatusCode::NOT_FOUND => return Err(ApiError::NoData),
                            _ => (),
                        }

                        let mut body = response.into_body().concat2();
                        let poll_result = body.poll();
                        mem::replace(self, ProcessRequestFuture::ProcessingResponse(body));
                        poll_result
                    },
                },
            ProcessRequestFuture::ProcessingResponse(body) => body.poll(),
        };

        match response_poll_result {
            Ok(Async::NotReady) => Ok(Async::NotReady),
            Err(err) => {
                error!("Error reading/processing request response {:?}", err);

                return Err(ApiError::Custom(err))
            },
            Ok(Async::Ready(chunk)) =>
                match str::from_utf8(&chunk) {
                    Ok(body) => {
                        trace!("Received response {}", body);

                        match R::handle(&body) {
                            Err(err) => {
                                error!("Error processing body: {:?}", err);

                                Err(err)
                            },
                            Ok(object) => Ok(Async::Ready(object)),
                        }
                    },
                    Err(err) => {
                        error!("Encoding error in response! {:?}", err);

                        Err(ApiError::UnexpectedFormat)
                    },
                },
        }
    }
}

impl<R: GdcfRequest + Handler> Action for ApiRequestAction<R> {
    type Error = ApiError;
    type Future = ProcessRequestFuture<R>;
    type Item = Response<R::Result>;

    fn run(&mut self) -> Self::Future {
        ProcessRequestFuture::WaitingForResponse(self.client.request(make_request::<R>(&self.encoded_request)), PhantomData)
    }
}

fn make_request<R: GdcfRequest + Handler>(encoded_request: &str) -> Request<Body> {
    let len = encoded_request.len();

    info!("Preparing request {} to {}", encoded_request, R::endpoint());

    let mut req = Request::new(Body::from(encoded_request.to_string()));

    *req.method_mut() = Method::POST;
    *req.uri_mut() = R::endpoint().parse().unwrap();
    req.headers_mut()
        .insert("Content-Type", HeaderValue::from_str("application/x-www-form-urlencoded").unwrap());
    req.headers_mut()
        .insert("Content-Length", HeaderValue::from_str(&len.to_string()).unwrap());

    req
}
