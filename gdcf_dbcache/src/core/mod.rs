use core::backend::Database;
use core::backend::Error;
use core::query::QueryPart;
use core::statement::PreparedStatement;
use std::fmt::Debug;
use core::statement::Preparation;

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

pub trait SqlExpression<DB: Database>: QueryPart<DB> {}

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
        (PreparedStatement::default(), vec!(self))
    }
}

impl<'a, DB: Database> QueryPart<DB> for AsSql<DB> + 'a {
    fn to_sql_unprepared(&self) -> String {
        self.as_sql_string()
    }

    fn to_sql<'b>(&'b self) -> Preparation<'b, DB> {
        (PreparedStatement::default(), vec!(self))
    }
}