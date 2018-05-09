use std::fmt::Debug;
use core::query::Query;
use core::AsSql;

#[cfg(feature = "pg")]
pub mod pg;

#[cfg(feature = "sqlite")]
pub mod sqlite;

#[cfg(feature = "mysql")]
pub mod mysql;

pub(crate) trait Database: Debug {
    type Types;
    type Error;

    fn prepare(idx: usize) -> String;

    fn execute<'a>(&'a self, query: &'a Query<'a, Self>) -> Result<(), Self::Error>
        where
            Self: Sized
    {
        let (stmt, params) = query.to_sql();
        self.execute_raw(stmt.to_statement(Self::prepare), &params)
    }

    fn execute_unprepared<'a>(&'a self, query: &'a Query<'a, Self>) -> Result<(), Self::Error>
        where
            Self: Sized
    {
        self.execute_raw(query.to_sql_unprepared(), &[])
    }

    fn execute_raw(&self, statement: String, params: &[&AsSql<Self>]) -> Result<(), Self::Error>;
}