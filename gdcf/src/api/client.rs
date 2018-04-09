use futures::Future;
use model::level::Level;
use api::request::level::LevelRequest;
use model::GDObject;
use std::str::Utf8Error;
use tokio_core::reactor::Handle;

#[derive(PartialEq, Debug)]
pub enum GDError {
    InternalServerError,
    ServersDown,
    Timeout,
    NoData,
    Unspecified,
    MalformedData(Utf8Error),
}

impl From<Utf8Error> for GDError {
    fn from(err: Utf8Error) -> Self {
        GDError::MalformedData(err)
    }
}

pub trait GDClient
{
    fn handle(&self) -> &Handle;

    fn level(&self, req: LevelRequest) -> Box<Future<Item=GDObject, Error=GDError> + 'static>;
}