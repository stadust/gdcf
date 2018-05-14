use core::AsSql;
use core::query::Query;
use core::query::select::Queryable;
use core::query::select::Row;
use std::error::Error as StdError;
use std::fmt::Debug;
use std::fmt::Display;
use std::fmt::Formatter;
use std::fmt::Error as FmtError;

#[cfg(feature = "pg")]
pub mod pg;

#[cfg(feature = "sqlite")]
pub mod sqlite;

#[cfg(feature = "mysql")]
pub mod mysql;

#[derive(Debug)]
pub(crate) enum Error<DB: Database> {
    Database(DB::Error),
    Conversion(String, &'static str),
}

pub(crate) trait Database: Debug + Sized {
    type Types: Debug;
    type Error: StdError;

    fn prepare(idx: usize) -> String;

    fn execute<'a>(&'a self, query: &'a Query<'a, Self>) -> Result<(), Error<Self>>
        where
            Self: Sized
    {
        let (stmt, params) = query.to_sql();
        self.execute_raw(stmt.to_statement(Self::prepare), &params)
    }

    fn execute_unprepared<'a>(&'a self, query: &'a Query<'a, Self>) -> Result<(), Error<Self>>
        where
            Self: Sized
    {
        self.execute_raw(query.to_sql_unprepared(), &[])
    }

    fn execute_raw(&self, statement: String, params: &[&AsSql<Self>]) -> Result<(), Error<Self>>;

    fn query<'a, T>(&'a self, query: &'a Query<'a, Self>) -> Result<Vec<T>, Error<Self>>
        where
            T: Queryable<Self>
    {
        let (stmt, params) = query.to_sql();
        let mut ts = Vec::new();

        for row in self.query_raw(stmt.to_statement(Self::prepare), &params)? {
            ts.push(T::from_row(&row, 0)?)
        }

        Ok(ts)
    }

    fn query_raw(&self, statement: String, params: &[&AsSql<Self>]) -> Result<Vec<Row<Self>>, Error<Self>>
        where
            Self: Sized;
}

impl<DB: Database> StdError for Error<DB> {
    fn description(&self) -> &str {
        match self {
            Error::Database(err) => err.description(),
            Error::Conversion(..) => "error while converting data from sql"
        }
    }
}

impl<DB: Database> Display for Error<DB> {
    fn fmt(&self, f: &mut Formatter) -> Result<(), FmtError> {
        match self {
            Error::Database(err) => write!(f, "{}", err),
            Error::Conversion(value, target) => write!(f, "Failed to convert {:?} to {}", value, target)
        }
    }
}