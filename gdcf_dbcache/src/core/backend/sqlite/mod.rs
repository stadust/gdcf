use crate::{
    core::{
        backend::{Database, Error},
        query::select::Row,
        AsSql,
    },
    resulter::Resulter,
};
use r2d2::Pool;
use r2d2_sqlite::SqliteConnectionManager;
use rusqlite::{types::ToSql, Error as DbError};
use std::path::Path;

mod convert;
mod query;

#[derive(Debug, Clone)]
pub struct Sqlite {
    pool: Pool<SqliteConnectionManager>,
}

impl Sqlite {
    pub(crate) fn memory() -> Result<Sqlite, Error<Sqlite>> {
        let manager = SqliteConnectionManager::memory();

        Ok(Sqlite { pool: Pool::new(manager)? })
    }

    pub(crate) fn path<P: AsRef<Path>>(path: P) -> Result<Sqlite, Error<Sqlite>> {
        let manager = SqliteConnectionManager::file(path);

        Ok(Sqlite { pool: Pool::new(manager)? })
    }
}

#[derive(Debug)]
pub enum SqliteTypes {
    Integer(i64),
    Real(f64),
    Text(String),
    Blob(Vec<u8>),
    Null,
}

impl Database for Sqlite {
    type Error = DbError;
    type Types = SqliteTypes;

    fn prepare(idx: usize) -> String {
        format!("?{}", idx)
    }

    fn execute_raw(&self, statement: String, params: &[&dyn AsSql<Self>]) -> Result<(), Error<Sqlite>> {
        let connection = self.pool.get()?;

        let comp = params.iter().map(|param| param.as_sql()).collect::<Vec<_>>();
        let values = comp.iter().map(|v| v as &dyn ToSql).collect::<Vec<_>>();

        connection.execute(&statement, &values[..])?;
        Ok(())
    }

    fn query_raw(&self, statement: String, params: &[&dyn AsSql<Self>]) -> Result<Vec<Row<Self>>, Error<Sqlite>>
    where
        Self: Sized,
    {
        let connection = self.pool.get()?;

        let comp: Vec<_> = params.iter().map(|param| param.as_sql()).collect();
        let values: Vec<_> = comp.iter().map(|v| v as &dyn ToSql).collect();

        let mut stmt = connection.prepare(&statement)?;

        let rows: Result<_, Vec<DbError>> = stmt
            .query_map(&values[..], |row| {
                let mut values = Vec::new();

                for i in 0.. {
                    match row.get_checked::<_, SqliteTypes>(i) {
                        Err(DbError::InvalidColumnIndex(..)) => break,
                        Err(err) => return Err(err),
                        Ok(value) => values.push(value),
                    }
                }

                Ok(Row::new(values))
            })?
            .flatten_results()
            .collect2();

        Ok(rows?)
    }
}

impl From<DbError> for Error<Sqlite> {
    fn from(db_err: DbError) -> Self {
        Error::Database(db_err)
    }
}
