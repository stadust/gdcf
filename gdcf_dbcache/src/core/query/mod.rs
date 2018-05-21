use core::{AsSql, backend::Database, statement::PreparedStatement};
use core::backend::Error;
pub  use self::insert::{Insert, Insertable};
pub  use self::select::{Select};
use std::fmt::Debug;

pub  mod condition;
pub  mod create;
pub  mod insert;
pub  mod select;

pub trait QueryPart<'a, DB: Database + 'a>: Debug {
    fn to_sql_unprepared(&self) -> String;

    fn to_sql(&self) -> (PreparedStatement, Vec<&'a AsSql<DB>>) {
        (self.to_sql_unprepared().into(), Vec::new())
    }
}

pub trait Query<'a, DB: Database + 'a>: QueryPart<'a, DB> {
    fn execute(&'a self, db: &'a DB) -> Result<(), Error<DB>>
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
