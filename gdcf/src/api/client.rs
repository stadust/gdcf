use futures::{Async, Future};

use crate::{api::request::Request, error::ApiError, Secondary};
use std::fmt::Debug;

pub trait ApiClient: Clone + Sized + Sync + Send + 'static {
    type Err: ApiError;
}

#[derive(Debug)]
pub enum Response<T> {
    Exact(T),
    More(T, Vec<Secondary>),
}

pub trait MakeRequest<R: Request>: ApiClient {
    type Future: Future<Item = Response<R::Result>, Error = Self::Err> + Send + 'static;

    fn make(&self, request: &R) -> Self::Future;
}
/*
enum FutureState<F: Future> {
    Pending(F),
    Done(Option<F::Item>),
}

/// A future for processing multiple requests at the same time.
///
/// This future preserves order, meaning the resulting vector will contain the items the futures
/// resolved to in the same order as the futures they originated from.
#[allow(missing_debug_implementations)]
pub struct MultiRequestFuture<R: Request, A: MakeRequest<R>> {
    futures: Vec<FutureState<A::Future>>,
}

// essentially copied from futures::future::JoinAll
impl<R: Request, A: MakeRequest<R>> Future for MultiRequestFuture<R, A> {
    type Error = A::Err;
    type Item = Response<Vec<Option<R::Result>>>;

    fn poll(&mut self) -> Result<Async<Self::Item>, Self::Error> {
        let mut all_done = true;

        for i in 0..self.futures.len() - 1 {
            match &mut self.futures[i] {
                FutureState::Pending(future) =>
                    match future.poll()? {
                        Async::NotReady => {
                            all_done = false;
                        },
                        Async::Ready(done) => self.futures[i] = FutureState::Done(Some(done)),
                    },
                FutureState::Done(_) => (),
            }
        }

        if all_done {
            let futures = std::mem::replace(&mut self.futures, Vec::new());
            let secondary_capacity = futures.iter().fold(0, |capacity, future| {
                match future {
                    FutureState::Pending(_) => unreachable!(),
                    FutureState::Done(response) =>
                        match response {
                            Some(Response::Exact(_)) | None => capacity,
                            Some(Response::More(_, secondary)) => capacity + secondary.len(),
                        },
                }
            });

            let mut response: Response<Vec<Option<R::Result>>> = if secondary_capacity == 0 {
                Response::Exact(Vec::with_capacity(futures.len()))
            } else {
                Response::More(Vec::with_capacity(futures.len()), Vec::with_capacity(secondary_capacity))
            };

            for future in futures {
                match future {
                    FutureState::Pending(_) => unreachable!(),
                    FutureState::Done(Some(Response::Exact(item))) =>
                        match response {
                            Response::Exact(ref mut prev_items) | Response::More(ref mut prev_items, _) => prev_items.push(Some(item)),
                        },
                    FutureState::Done(Some(Response::More(item, mut secondaries))) =>
                        match response {
                            Response::Exact(_) => unreachable!(),
                            Response::More(ref mut prev_items, ref mut prev_secondaries) => {
                                prev_items.push(Some(item));
                                prev_secondaries.extend(secondaries)
                            },
                        },
                    FutureState::Done(None) =>
                        match response {
                            Response::Exact(ref mut items) | Response::More(ref mut items, _) => items.push(None),
                        },
                }
            }

            Ok(Async::Ready(response))
        } else {
            Ok(Async::NotReady)
        }
    }
}

impl<R: Request, A: MakeRequest<R>> MultiRequestFuture<R, A> {
    fn new(client: A, requests: &[Option<R>]) -> Self {
        Self {
            futures: requests
                .iter()
                .map(|maybe_req| {
                    match maybe_req {
                        Some(request) => FutureState::Pending(client.make(request)),
                        None => FutureState::Done(None),
                    }
                })
                .collect(),
        }
    }
}

impl<R: Request, A: MakeRequest<R>> MakeRequest<MultiRequest<R>> for A {
    type Future = MultiRequestFuture<R, A>;

    fn make(&self, requests: &MultiRequest<R>) -> Self::Future {
        MultiRequestFuture::new(self.clone(), &requests.0)
    }
}
*/
