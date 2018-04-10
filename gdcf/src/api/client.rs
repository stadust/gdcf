use futures::Future;

use tokio_core::reactor::Handle;

use model::GDObject;

use api::request::level::LevelRequest;
use api::GDError;

pub trait GDClient
{
    fn handle(&self) -> &Handle;

    fn level(&self, req: LevelRequest) -> Box<Future<Item=GDObject, Error=GDError> + 'static>;
}