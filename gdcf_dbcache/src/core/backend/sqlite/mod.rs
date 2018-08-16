use core::AsSql;
use core::backend::Database;
use core::backend::Error;
use core::query::select::Row;
use rusqlite::Error as DbError;

mod convert;

#[derive(Debug)]
pub struct Sqlite;

#[derive(Debug)]
pub enum SqliteTypes {
    Integer(i64),
    Real(f64),
    Text(String),
    Blob(Vec<u8>),
    Null,
}

impl Database for Sqlite {
    type Types = SqliteTypes;
    type Error = DbError;

    fn prepare(idx: usize) -> String {
        format!("?{}", idx)
    }

    fn execute_raw(&self, statement: String, params: &[&dyn AsSql<Self>]) -> Result<(), Error<Sqlite>> {
        unimplemented!()
    }

    fn query_raw(&self, statement: String, params: &[&dyn AsSql<Self>]) -> Result<Vec<Row<Self>>, Error<Sqlite>>
        where
            Self: Sized
    {
        unimplemented!()
    }
}
