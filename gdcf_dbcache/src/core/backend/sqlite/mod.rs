use core::AsSql;
use core::backend::Database;
use core::backend::Error;
use core::query::select::Row;
use resulter::Resulter;
use rusqlite::Connection;
use rusqlite::Error as DbError;
use rusqlite::types::ToSql;
use std::path::Path;

mod convert;
mod query;

#[derive(Debug)]
pub struct Sqlite {
    connection: Connection
}

impl Sqlite {
    pub(crate) fn memory() -> Sqlite {
        Sqlite {
            connection: Connection::open_in_memory().expect("Failure to create in-memory sqlite database")
        }
    }

    pub(crate) fn path<P: AsRef<Path>>(path: P) -> Sqlite {
        Sqlite {
            connection: Connection::open(path).expect("Yeah, that didn't work")
        }
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
    type Types = SqliteTypes;
    type Error = DbError;

    fn prepare(idx: usize) -> String {
        format!("?{}", idx)
    }

    fn execute_raw(&self, statement: String, params: &[&dyn AsSql<Self>]) -> Result<(), Error<Sqlite>> {
        let comp = params.into_iter().map(|param| param.as_sql()).collect::<Vec<_>>();
        let values = comp.iter().map(|v| v as &dyn ToSql).collect::<Vec<_>>();

        self.connection.execute(&statement, &values[..])?;
        Ok(())
    }

    fn query_raw(&self, statement: String, params: &[&dyn AsSql<Self>]) -> Result<Vec<Row<Self>>, Error<Sqlite>>
        where
            Self: Sized
    {
        let comp: Vec<_> = params.into_iter().map(|param| param.as_sql()).collect();
        let values: Vec<_> = comp.iter().map(|v| v as &dyn ToSql).collect();

        let mut stmt = self.connection.prepare(&statement)?;

        let rows: Result<_, Vec<DbError>> = stmt.query_map(&values[..], |row| {
            let mut values = Vec::new();

            for i in 0.. {
                match row.get_checked::<_, SqliteTypes>(i) {
                    Err(DbError::InvalidColumnIndex(..)) => break,
                    Err(err) => return Err(err),
                    Ok(value) => values.push(value)
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