use futures::Future;

use api::request::{LevelsRequest, LevelRequest};
use api::response::ProcessedResponse;

use std::error::Error;
use error::ApiError;

pub type ApiFuture<E> = Box<dyn Future<Item=ProcessedResponse, Error=ApiError<E>> + 'static>;

pub trait ApiClient: Sized {
    type Err: Error + 'static;

    fn level(&self, req: &LevelRequest) -> ApiFuture<Self::Err>;
    fn levels(&self, req: &LevelsRequest) -> ApiFuture<Self::Err>;

    fn spawn<F>(&self, f: F)
        where
            F: Future<Item=(), Error=()> + 'static;
}
