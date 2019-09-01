use futures::Future;

use crate::{api::request::Request, error::ApiError, Secondary};

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

    fn make(&self, request: &R) -> Self::Future; //ApiFuture<R, Self::Err>;
}
