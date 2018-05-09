use core::{AsSql, Database, query::Insertable, table::{FieldValue, SetField}};
use postgres::{Connection, Error, types::ToSql as ToPgSql};
use self::types::PgTypes;
use std::error;

mod condition;
mod types;
mod query;

#[derive(Debug)]
pub(crate) struct Pg {
    conn: Connection
}

impl Database for Pg {
    type Types = PgTypes;
    type Error = Error;

    fn prepare(idx: usize) -> String {
        format!("${}", idx)
    }

    fn execute_raw(&self, statement: String, params: &[&AsSql<Self>]) -> Result<(), <Self as Database>::Error> {
        let comp: Vec<_> = params.into_iter().map(|param| param.as_sql()).collect();
        let values: Vec<_> = comp.iter().map(|v| v as &ToPgSql).collect();

        self.conn.execute(&statement, &values[..])?;
        Ok(())
    }
}