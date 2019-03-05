use futures::Future;

use api::request::{LevelRequest, LevelsRequest};
use model::GDObject;

use api::request::user::UserRequest;
use error::ApiError;
use failure::Fail;

pub type ApiFuture<E> = Box<dyn Future<Item = Vec<GDObject>, Error = ApiError<E>> + Send + 'static>;

pub trait ApiClient: Clone + Sized + Sync + Send + 'static {
    type Err: Fail;

    fn level(&self, req: LevelRequest) -> ApiFuture<Self::Err>;
    fn levels(&self, req: LevelsRequest) -> ApiFuture<Self::Err>;

    fn user(&self, req: UserRequest) -> ApiFuture<Self::Err>;
}
