use core::AsSql;
use core::query::Query;
use core::query::select::Queryable;
use core::query::select::Row;
use std::error::Error as StdError;
use std::fmt::Debug;
use std::fmt::Display;
use std::fmt::Error as FmtError;
use std::fmt::Formatter;

#[cfg(feature = "pg")]
pub mod pg;

#[cfg(feature = "sqlite")]
pub mod sqlite;

#[cfg(feature = "mysql")]
pub mod mysql;

#[derive(Debug)]
pub enum Error<DB: Database> {
    Database(DB::Error),

    /// Conversion from a row item the expected Rust datatype failed
    Conversion(String, &'static str),

    /// The query passed to `query_one` didn't yield and rows
    NoResult,

    /// The query passed to `query_on` yieled more than one row
    TooManyRows,
}

pub trait Database: Debug + Sized {
    type Types: Debug;
    type Error: StdError;

    fn prepare(idx: usize) -> String;

    fn execute(&self, query: &dyn Query<Self>) -> Result<(), Error<Self>>
        where
            Self: Sized
    {
        let (stmt, params) = query.to_sql();
        self.execute_raw(stmt.to_statement(Self::prepare), &params)
    }

    fn execute_unprepared(&self, query: &dyn Query<Self>) -> Result<(), Error<Self>>
        where
            Self: Sized
    {
        self.execute_raw(query.to_sql_unprepared(), &[])
    }

    fn execute_raw(&self, statement: String, params: &[&dyn AsSql<Self>]) -> Result<(), Error<Self>>;

    fn query_one<T>(&self, query: &dyn Query<Self>) -> Result<T, Error<Self>>
        where
            T: Queryable<Self>
    {
        let mut result = self.query(query)?;

        if result.is_empty() {
            Err(Error::NoResult)
        } else if result.len() > 1 {
            Err(Error::TooManyRows)
        } else {
            Ok(result.remove(0))
        }
    }
    fn query_one_unprepared<T>(&self, query: &dyn Query<Self>) -> Result<T, Error<Self>>
        where
            T: Queryable<Self>
    {
        let mut result = self.query_unprepared(query)?;

        if result.is_empty() {
            Err(Error::NoResult)
        } else if result.len() > 1 {
            Err(Error::TooManyRows)
        } else {
            Ok(result.remove(0))
        }
    }

    fn query<T>(&self, query: &dyn Query<Self>) -> Result<Vec<T>, Error<Self>>
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

    fn query_unprepared<T>(&self, query: &dyn Query<Self>) -> Result<Vec<T>, Error<Self>>
        where
            T: Queryable<Self>
    {
        let mut ts = Vec::new();

        for row in self.query_raw(query.to_sql_unprepared(), &[])? {
            ts.push(T::from_row(&row, 0)?)
        }

        Ok(ts)
    }

    fn query_raw(&self, statement: String, params: &[&dyn AsSql<Self>]) -> Result<Vec<Row<Self>>, Error<Self>>
        where
            Self: Sized;
}

impl<DB: Database> StdError for Error<DB> {
    fn description(&self) -> &str {
        match self {
            Error::Database(err) => err.description(),
            Error::Conversion(..) => "error while converting data from sql",
            Error::NoResult => "query yielded no result",
            Error::TooManyRows => "query yielded too many rows"
        }
    }
}

impl<DB: Database> Display for Error<DB> {
    fn fmt(&self, f: &mut Formatter) -> Result<(), FmtError> {
        match self {
            Error::Database(err) => write!(f, "{}", err),
            Error::Conversion(value, target) => write!(f, "Failed to convert {:?} to {}", value, target),
            e => write!(f, "{}", e.description())
        }
    }
}