use futures::Future;

use crate::{
    api::request::{user::UserRequest, LevelRequest, LevelsRequest, Request},
    error::ApiError,
    GDObject,
};

pub type ApiFuture<E> = Box<dyn Future<Item = Vec<GDObject>, Error = E> + Send + 'static>;

pub trait ApiClient: Clone + Sized + Sync + Send + 'static {
    type Err: ApiError;

    fn level(&self, req: LevelRequest) -> ApiFuture<Self::Err>;
    fn levels(&self, req: LevelsRequest) -> ApiFuture<Self::Err>;

    fn user(&self, req: UserRequest) -> ApiFuture<Self::Err>;
}

#[derive(Debug)]
pub enum Response<T> {
    Exact(T),
    More(T, Vec<GDObject>),
}

pub trait MakeRequest<R: Request>: ApiClient {
    fn make(&self, request: &R) -> Box<dyn Future<Item = Response<R::Result>, Error = Self::Err> + Send + 'static>;
}
