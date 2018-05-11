use core::AsSql;
use core::query::Query;
use core::query::select::Queryable;
use core::query::select::Row;
use std::fmt::Debug;

#[cfg(feature = "pg")]
pub mod pg;

#[cfg(feature = "sqlite")]
pub mod sqlite;

#[cfg(feature = "mysql")]
pub mod mysql;

pub(crate) trait Database: Debug + Sized {
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

    fn query<'a, T>(&'a self, query: &'a Query<'a, Self>) -> Result<Vec<T>, Self::Error>
        where
            T: Queryable<Self>
    {
        let (stmt, params) = query.to_sql();
        let mut ts = Vec::new();

        for row in self.query_raw(stmt.to_statement(Self::prepare), &params)? {
            ts.push(T::from_row(&row))
        }

        Ok(ts)
    }

    fn query_raw(&self, statement: String, params: &[&AsSql<Self>]) -> Result<Vec<Row<Self>>, Self::Error>
        where
            Self: Sized;
}