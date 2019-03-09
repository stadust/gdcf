pub use self::{
    insert::{Insert, Insertable},
    select::Select,
};
use crate::core::{
    backend::{Database, Error},
    QueryPart,
};

pub mod condition;
pub mod create;
pub mod delete;
pub mod insert;
pub mod select;

pub trait Query<DB: Database>: QueryPart<DB> {
    fn execute(&self, db: &DB) -> Result<(), Error<DB>>
    where
        Self: Sized,
    {
        db.execute(self)
    }

    fn execute_unprepared(&self, db: &DB) -> Result<(), Error<DB>>
    where
        Self: Sized,
    {
        db.execute_unprepared(self)
    }
}
