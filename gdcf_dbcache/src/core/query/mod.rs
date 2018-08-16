use core::backend::{Database, Error};
pub use self::insert::{Insert, Insertable};
pub use self::select::Select;
use core::QueryPart;

pub mod condition;
pub mod create;
pub mod insert;
pub mod select;
pub mod delete;

pub trait Query<DB: Database>: QueryPart<DB> {
    fn execute(&self, db: &DB) -> Result<(), Error<DB>>
        where
            Self: Sized
    {
        db.execute(self)
    }

    fn execute_unprepared(&self, db: &DB) -> Result<(), Error<DB>>
        where
            Self: Sized
    {
        db.execute_unprepared(self)
    }
}