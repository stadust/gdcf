use super::Error;
use crate::core::{backend::Database, query::select::Row, AsSql};
use chrono::NaiveDateTime;
use postgres::{types::ToSql as ToPgSql, Error as PgError};
use r2d2::Pool;
use r2d2_postgres::{PostgresConnectionManager, TlsMode};

mod convert;
mod query;

#[derive(Debug, Clone)]
pub struct Pg {
    pool: Pool<PostgresConnectionManager>,
}

#[derive(Debug)]
pub enum PgTypes {
    SmallInteger(i16),
    Integer(i32),
    BigInteger(i64),
    Text(String),
    Float(f32),
    Double(f64),
    Boolean(bool),
    Timestamp(NaiveDateTime),
    Bytes(Vec<u8>),
    Null,
}

impl Pg {
    pub fn new(url: &str, tls: TlsMode) -> Result<Pg, Error<Pg>> {
        let manager = PostgresConnectionManager::new(url, tls)?;

        Ok(Pg {
            pool: Pool::new(manager).unwrap(),
        })
    }
}

impl Database for Pg {
    type Error = PgError;
    type Types = PgTypes;

    fn prepare(idx: usize) -> String {
        format!("${}", idx)
    }

    fn execute_raw(&self, statement: String, params: &[&dyn AsSql<Self>]) -> Result<(), Error<Pg>> {
        let connection = self.pool.get()?;

        let comp: Vec<_> = params.iter().map(|param| param.as_sql()).collect();
        let values: Vec<_> = comp.iter().map(|v| v as &dyn ToPgSql).collect();

        connection.execute(&statement, &values[..])?;
        Ok(())
    }

    fn query_raw(&self, statement: String, params: &[&dyn AsSql<Self>]) -> Result<Vec<Row<Pg>>, Error<Pg>>
    where
        Self: Sized,
    {
        let connection = self.pool.get()?;

        let comp: Vec<_> = params.iter().map(|param| param.as_sql()).collect();
        let values: Vec<_> = comp.iter().map(|v| v as &dyn ToPgSql).collect();

        let mut rows = Vec::new();

        for row in connection.query(&statement, &values)?.iter() {
            let mut values = Vec::new();

            for i in 0..row.len() {
                values.push(row.get_opt(i).unwrap()?)
            }

            rows.push(Row::new(values));
        }

        Ok(rows)
    }
}

impl From<PgError> for Error<Pg> {
    fn from(pg: PgError) -> Self {
        Error::Database(pg)
    }
}
