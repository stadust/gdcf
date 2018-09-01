use core::{
    query::{
        select::{Queryable, Row},
        Query,
    },
    AsSql,
};
use std::{
    error::Error as StdError,
    fmt::{Debug, Display, Error as FmtError, Formatter},
};

#[cfg(feature = "pg")]
pub mod pg;

#[cfg(feature = "sqlite")]
pub mod sqlite;

#[cfg(feature = "mysql")]
pub mod mysql;

mod util;

// TODO: during FromSql<DB> and AsSql<DB> we are excessively copying things. We
// probably either wanna work with references more, or take ownership more

// TODO: error handling when creating connections

// TODO: transaction support

#[derive(Debug)]
pub enum Error<DB: Database> {
    /// Database specific error
    Database(DB::Error),

    /// Many database specific errros
    MultipleDatabase(Vec<DB::Error>),

    /// Conversion from a row item the expected Rust datatype failed
    Conversion(String, &'static str),

    /// The query passed to `query_one` didn't yield any rows
    NoResult,

    /// The query passed to `query_one` yielded more than one row
    TooManyRows,
}

pub trait Database: Debug + Sized {
    type Types: Display;
    type Error: StdError + Send + 'static;

    fn prepare(idx: usize) -> String;

    fn execute(&self, query: &dyn Query<Self>) -> Result<(), Error<Self>>
    where
        Self: Sized,
    {
        trace!("Executing query {}", query.to_raw_sql());

        let (stmt, params) = query.to_sql();
        self.execute_raw(stmt.to_statement(Self::prepare), &params)
    }

    fn execute_unprepared(&self, query: &dyn Query<Self>) -> Result<(), Error<Self>>
    where
        Self: Sized,
    {
        self.execute_raw(query.to_raw_sql(), &[])
    }

    fn execute_raw(&self, statement: String, params: &[&dyn AsSql<Self>]) -> Result<(), Error<Self>>;

    fn query_one<T>(&self, query: &dyn Query<Self>) -> Result<T, Error<Self>>
    where
        T: Queryable<Self>,
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

    fn query_one_row(&self, query: &dyn Query<Self>) -> Result<Row<Self>, Error<Self>> {
        let mut result = self.query_rows(query)?;

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
        T: Queryable<Self>,
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
        T: Queryable<Self>,
    {
        trace!("Executing query {}", query.to_raw_sql());

        let (stmt, params) = query.to_sql();
        let mut ts = Vec::new();

        for row in self.query_raw(stmt.to_statement(Self::prepare), &params)? {
            ts.push(T::from_row(&row, 0)?)
        }

        Ok(ts)
    }

    fn query_rows(&self, query: &dyn Query<Self>) -> Result<Vec<Row<Self>>, Error<Self>> {
        trace!("Executing query {}", query.to_raw_sql());

        let (stmt, params) = query.to_sql();

        self.query_raw(stmt.to_statement(Self::prepare), &params)
    }

    fn query_unprepared<T>(&self, query: &dyn Query<Self>) -> Result<Vec<T>, Error<Self>>
    where
        T: Queryable<Self>,
    {
        let mut ts = Vec::new();

        for row in self.query_raw(query.to_raw_sql(), &[])? {
            ts.push(T::from_row(&row, 0)?)
        }

        Ok(ts)
    }

    fn query_raw(&self, statement: String, params: &[&dyn AsSql<Self>]) -> Result<Vec<Row<Self>>, Error<Self>>
    where
        Self: Sized;
}

impl<DB: Database> StdError for Error<DB> {}

impl<DB: Database> Display for Error<DB> {
    fn fmt(&self, f: &mut Formatter) -> Result<(), FmtError> {
        match self {
            Error::Database(err) => write!(f, "{}", err),
            Error::Conversion(value, target) => write!(f, "Failed to convert {:?} to {}", value, target),
            e => write!(f, "{}", e),
        }
    }
}

impl<DB: Database> From<Vec<DB::Error>> for Error<DB> {
    fn from(errs: Vec<DB::Error>) -> Self {
        Error::MultipleDatabase(errs)
    }
}
