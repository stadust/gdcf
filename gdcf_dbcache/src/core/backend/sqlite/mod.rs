use core::AsSql;
use core::backend::Database;
use core::query::select::Row;

use rusqlite::Error as DbError;

#[derive(Debug)]
pub struct Sqlite;


impl Database for Sqlite {
    type Types = ();
    type Error = DbError;

    fn prepare(idx: usize) -> String {
        format!("?{}", idx)
    }

    fn execute_raw(&self, statement: String, params: &[&dyn AsSql<Self>]) -> Result<(), <Self as Database>::Error> {
        unimplemented!()
    }

    fn query_raw(&self, statement: String, params: &[&dyn AsSql<Self>]) -> Result<Vec<Row<Self>>, <Self as Database>::Error> where
        Self: Sized {
        unimplemented!()
    }
}
