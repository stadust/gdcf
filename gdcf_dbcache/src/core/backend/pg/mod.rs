use core::{AsSql, backend::Database};
use core::query::select::Row;
use postgres::{Connection, Error as PgError, types::ToSql as ToPgSql};
use self::convert::PgTypes;
use super::Error;

mod condition;
mod convert;
mod query;
mod types;
mod constraint;

#[derive(Debug)]
pub  struct Pg {
    conn: Connection
}

impl Database for Pg {
    type Types = PgTypes;
    type Error = PgError;

    fn prepare(idx: usize) -> String {
        format!("${}", idx)
    }

    fn execute_raw(&self, statement: String, params: &[&AsSql<Self>]) -> Result<(), Error<Pg>> {
        let comp: Vec<_> = params.into_iter().map(|param| param.as_sql()).collect();
        let values: Vec<_> = comp.iter().map(|v| v as &ToPgSql).collect();

        self.conn.execute(&statement, &values[..])?;
        Ok(())
    }

    fn query_raw(&self, statement: String, params: &[&AsSql<Self>]) -> Result<Vec<Row<Pg>>, Error<Pg>>
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

impl From<PgError> for Error<Pg> {
    fn from(pg: PgError) -> Self {
        Error::Database(pg)
    }
}