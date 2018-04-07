use futures::Future;
use model::level::Level;
use api::request::level::LevelRequest;
use model::GDObject;

#[derive(PartialOrd, PartialEq, Debug)]
pub enum GDError {
    ServersDown,
    NoResponse,
    NoData,
}

pub trait GDClient
{
    type Fut: Future<Item=GDObject, Error=GDError> + Send + 'static;

    fn level(&self, req: LevelRequest) -> Self::Fut;
}