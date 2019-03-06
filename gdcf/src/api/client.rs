use futures::Future;

use crate::{
    api::request::{user::UserRequest, LevelRequest, LevelsRequest},
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
