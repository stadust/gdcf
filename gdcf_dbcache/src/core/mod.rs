use core::backend::Database;
use core::backend::Error;
use core::query::QueryPart;
use core::statement::Preparation;
use core::statement::PreparedStatement;
use std::fmt::Debug;

#[macro_use]
pub mod macros;
pub mod query;
pub mod table;
pub mod backend;
pub mod statement;
pub mod types;

pub trait AsSql<DB: Database>: Debug {
    fn as_sql(&self) -> DB::Types;
    fn as_sql_string(&self) -> String;
}

pub trait SqlExpr<DB: Database>: QueryPart<DB> {}

impl<'a, T: 'a, DB: Database> AsSql<DB> for &'a T
    where
        T: AsSql<DB>
{
    fn as_sql(&self) -> <DB as Database>::Types {
        (*self).as_sql()
    }

    fn as_sql_string(&self) -> String {
        (*self).as_sql_string()
    }
}

pub trait FromSql<DB: Database> {
    fn from_sql(sql: &DB::Types) -> Result<Self, Error<DB>>
        where
            Self: Sized;
}

impl<DB: Database, T> QueryPart<DB> for T
    where
        T: AsSql<DB>
{
    fn to_sql_unprepared(&self) -> String {
        self.as_sql_string()
    }

    fn to_sql<'a>(&'a self) -> Preparation<'a, DB> {
        (PreparedStatement::placeholder(), vec!(self))
    }
}

// Impl for trait objects
// I honestly have no clue what that lifetime does, but without it seems to assume an
// impl only for AsSql<DB> objects with a static lifetime
impl<'a, DB: Database> QueryPart<DB> for dyn AsSql<DB> + 'a {
    fn to_sql_unprepared(&self) -> String {
        self.as_sql_string()
    }

    fn to_sql<'b>(&'b self) -> Preparation<'b, DB> {
        (PreparedStatement::placeholder(), vec!(self))
    }
}

impl<DB: Database, T> SqlExpr<DB> for T
    where
        T: AsSql<DB>
{}