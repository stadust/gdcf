use core::{AsSql, backend::Database};
use core::query::select::Row;
use postgres::{Connection, Error, types::ToSql as ToPgSql};
use self::types::PgTypes;

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

    fn query_raw(&self, statement: String, params: &[&AsSql<Self>]) -> Result<Vec<Row<Pg>>, Error>
        where
            Self: Sized
    {
        let comp: Vec<_> = params.into_iter().map(|param| param.as_sql()).collect();
        let values: Vec<_> = comp.iter().map(|v| v as &ToPgSql).collect();

        let mut rows = Vec::new();

        for row in self.conn.query(&statement, &values)?.iter() {
            rows.push(Row::new((0..row.len()).map(|i| row.get(i)).collect()))
        }

        Ok(rows)
    }
}