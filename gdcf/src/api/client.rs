use futures::Future;

use api::request::{LevelsRequest, LevelRequest};
use api::response::ProcessedResponse;
use api::GDError;

use model::RawObject;
use api::request::MakeRequest;

pub type ApiFuture = Box<Future<Item=ProcessedResponse, Error=GDError> + 'static>;

pub trait ApiClient: Sized {
    fn level(&self, req: &LevelRequest) -> ApiFuture;
    fn levels(&self, req: &LevelsRequest) -> ApiFuture;

    fn spawn<F>(&self, f: F)
        where
            F: Future<Item=(), Error=()> + 'static;
}
