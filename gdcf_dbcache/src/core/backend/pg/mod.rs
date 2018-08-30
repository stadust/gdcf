use chrono::NaiveDateTime;
use core::{AsSql, backend::Database};
use core::query::select::Row;
use postgres::{Connection, Error as PgError, types::ToSql as ToPgSql};
use postgres::TlsMode;
use super::Error;

mod convert;
mod query;

#[derive(Debug)]
pub struct Pg {
    connection: Connection
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
    pub fn new(url: &str, tls: TlsMode) -> Pg {
        Pg {
            connection: Connection::connect(url, tls).unwrap()
        }
    }
}

impl Database for Pg {
    type Types = PgTypes;
    type Error = PgError;

    fn prepare(idx: usize) -> String {
        format!("${}", idx)
    }

    fn execute_raw(&self, statement: String, params: &[&dyn AsSql<Self>]) -> Result<(), Error<Pg>> {
        let comp: Vec<_> = params.into_iter().map(|param| param.as_sql()).collect();
        let values: Vec<_> = comp.iter().map(|v| v as &dyn ToPgSql).collect();

        self.connection.execute(&statement, &values[..])?;
        Ok(())
    }

    fn query_raw(&self, statement: String, params: &[&dyn AsSql<Self>]) -> Result<Vec<Row<Pg>>, Error<Pg>>
        where
            Self: Sized
    {
        let comp: Vec<_> = params.into_iter().map(|param| param.as_sql()).collect();
        let values: Vec<_> = comp.iter().map(|v| v as &dyn ToPgSql).collect();

        let mut rows = Vec::new();

        for row in self.connection.query(&statement, &values)?.iter() {
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