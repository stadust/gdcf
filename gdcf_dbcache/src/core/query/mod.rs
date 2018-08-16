//use core::{backend::Database, statement::Preparation};
//use core::AsSql;
use core::backend::{Database, Error};
//use core::statement::PreparedStatement;
pub use self::insert::{Insert, Insertable};
pub use self::select::Select;
//use core::AsRawSql;
//use std::fmt::Debug;
//use core::statement::Prepare;
use core::QueryPart;

pub mod condition;
pub mod create;
pub mod insert;
pub mod select;
pub mod delete;

/*impl<DB: Database, T: AsSql<DB>> QueryPart<DB> for T {
    fn to_sql(&self) -> Preparation<DB> {
        (PreparedStatement::placeholder(), vec!(self))
    }
}*/
/*
impl<DB: Database, T: QueryPart<DB>> AsRawSql<DB> for T {
    fn as_raw_sql(&self) -> String {
        self.to_sql().unprepared()
    }
}

impl<'a, DB: Database> AsRawSql<DB> for dyn QueryPart<DB> + 'a {
    fn as_raw_sql(&self) -> String {
        self.to_sql().unprepared()
    }
}*/

// Impl for trait objects
// I honestly have no clue what that lifetime does, but without it seems to assume an
// impl only for AsSql<DB> objects with a static lifetime
/*impl<'a, DB: Database> QueryPart<DB> for dyn AsSql<DB> + 'a {
    fn to_sql(&self) -> Preparation<DB> {
        (PreparedStatement::placeholder(), vec!(self))
    }
}*/

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
/*
impl<'a, DB: Database> AsRawSql<DB> for dyn Query<DB> + 'a {
    fn as_raw_sql(&self) -> String {
        self.to_sql().unprepared()
    }
}*/