use futures::Future;

use api::{
    request::{LevelRequest, LevelsRequest},
    response::ProcessedResponse,
};

//use api::request::user::UserRequest;
use error::ApiError;
use std::error::Error;

pub type ApiFuture<E> = Box<dyn Future<Item = ProcessedResponse, Error = ApiError<E>> + Send + 'static>;

pub trait ApiClient: Sized + Send + 'static {
    type Err: Error + Send + 'static;

    fn level(&self, req: LevelRequest) -> ApiFuture<Self::Err>;
    fn levels(&self, req: LevelsRequest) -> ApiFuture<Self::Err>;

    //fn user(&self, req: UserRequest) -> ApiFuture<Self::Err>;
}
