use core::AsSql;
use core::backend::Database;
use core::query::select::Row;

#[derive(Debug)]
pub(crate) struct Sqlite;


impl Database for Sqlite {
    type Types = ();
    type Error = ();

    fn prepare(idx: usize) -> String {
        unimplemented!()
    }

    fn execute_raw(&self, statement: String, params: &[&AsSql<Self>]) -> Result<(), <Self as Database>::Error> {
        unimplemented!()
    }

    fn query_raw(&self, statement: String, params: &[&AsSql<Self>]) -> Result<Vec<Row<Self>>, <Self as Database>::Error> where
        Self: Sized {
        unimplemented!()
    }
}