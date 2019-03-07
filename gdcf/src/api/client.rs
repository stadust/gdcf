use futures::Future;

use crate::{api::request::Request, error::ApiError, GDObject};

pub type ApiFuture<R, E> = Box<dyn Future<Item = Response<<R as Request>::Result>, Error = E> + Send + 'static>;

pub trait ApiClient: Clone + Sized + Sync + Send + 'static {
    type Err: ApiError;
}

#[derive(Debug)]
pub enum Response<T> {
    Exact(T),
    More(T, Vec<GDObject>),
}

pub trait MakeRequest<R: Request>: ApiClient {
    fn make(&self, request: R) -> ApiFuture<R, Self::Err>;
}
