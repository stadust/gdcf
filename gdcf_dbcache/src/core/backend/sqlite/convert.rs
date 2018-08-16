use core::AsSql;
use core::backend::Database;
use core::backend::Error;
use core::backend::sqlite::Sqlite;
use core::backend::sqlite::SqliteTypes;
use core::FromSql;
use rusqlite::Error as SqliteError;
use rusqlite::types::FromSql as SqliteFromSql;
use rusqlite::types::FromSqlError;
use rusqlite::types::ToSql as SqliteToSql;
use rusqlite::types::ToSqlOutput;
use rusqlite::types::Value;
use rusqlite::types::ValueRef;
use std::fmt::{Display, Error as FmtError, Formatter};

impl Display for SqliteTypes {
    fn fmt(&self, f: &mut Formatter) -> Result<(), FmtError> {
        match self {
            SqliteTypes::Integer(i) => write!(f, "{}", i),
            SqliteTypes::Real(d) => write!(f, "{}", d),
            SqliteTypes::Text(s) => write!(f, "\"{}\"", s),
            SqliteTypes::Null => write!(f, "NULL"),
            SqliteTypes::Blob(_) => write!(f, "<binary data>")
        }
    }
}

impl SqliteFromSql for SqliteTypes {
    fn column_result(value: ValueRef) -> Result<Self, FromSqlError> {
        Ok(match value {
            ValueRef::Integer(i) => SqliteTypes::Integer(i),
            ValueRef::Real(r) => SqliteTypes::Real(r),
            ValueRef::Text(s) => SqliteTypes::Text(s.to_string()),
            ValueRef::Blob(blob) => SqliteTypes::Blob(blob.to_vec()),
            ValueRef::Null => SqliteTypes::Null
        })
    }
}

impl SqliteToSql for SqliteTypes {
    fn to_sql(&self) -> Result<ToSqlOutput, SqliteError> {
        Ok(match self {
            SqliteTypes::Integer(i) => ToSqlOutput::Owned(Value::Integer(*i)),
            SqliteTypes::Real(r) => ToSqlOutput::Owned(Value::Real(*r)),
            SqliteTypes::Text(s) => ToSqlOutput::Borrowed(ValueRef::Text(s)),
            SqliteTypes::Blob(b) => ToSqlOutput::Borrowed(ValueRef::Blob(b)),
            SqliteTypes::Null => ToSqlOutput::Owned(Value::Null)
        })
    }
}

as_sql_cast!(Sqlite, i8, i64, SqliteTypes::Integer);
as_sql_cast!(Sqlite, u8, i64, SqliteTypes::Integer);
as_sql_cast!(Sqlite, i16, i64, SqliteTypes::Integer);
as_sql_cast!(Sqlite, u16, i64, SqliteTypes::Integer);
as_sql_cast!(Sqlite, i32, i64, SqliteTypes::Integer);
as_sql_cast!(Sqlite, u32, i64, SqliteTypes::Integer);
as_sql_cast!(Sqlite, i64, i64, SqliteTypes::Integer);
as_sql_cast!(Sqlite, u64, i64, SqliteTypes::Integer);

as_sql_cast!(Sqlite, bool, i64, SqliteTypes::Integer);

as_sql_cast!(Sqlite, f32, f64, SqliteTypes::Real);
as_sql_cast!(Sqlite, f64, f64, SqliteTypes::Real);

impl<'a> AsSql<Sqlite> for &'a str {
    fn as_sql(&self) -> SqliteTypes {
        SqliteTypes::Text(self.to_string())
    }
}

impl AsSql<Sqlite> for String {
    fn as_sql(&self) -> SqliteTypes {
        SqliteTypes::Text(self.clone())
    }
}

impl AsSql<Sqlite> for Vec<u8> {
    fn as_sql(&self) -> SqliteTypes {
        SqliteTypes::Blob(self.clone())
    }
}

impl<T> AsSql<Sqlite> for Option<T>
    where
        T: AsSql<Sqlite>
{
    fn as_sql(&self) -> SqliteTypes {
        match self {
            None => SqliteTypes::Null,
            Some(t) => t.as_sql()
        }
    }
}

from_sql_cast!(Sqlite, u8, SqliteTypes::Integer);
from_sql_cast!(Sqlite, i8, SqliteTypes::Integer);
from_sql_cast!(Sqlite, i16, SqliteTypes::Integer);
from_sql_cast!(Sqlite, u16, SqliteTypes::Integer);
from_sql_cast!(Sqlite, i32, SqliteTypes::Integer);
from_sql_cast!(Sqlite, u32, SqliteTypes::Integer);
from_sql_cast!(Sqlite, i64, SqliteTypes::Integer);
from_sql_cast!(Sqlite, u64, SqliteTypes::Integer);

from_sql_cast!(Sqlite, f32, SqliteTypes::Real);
from_sql_cast!(Sqlite, f64, SqliteTypes::Real);

impl<T> FromSql<Sqlite> for Option<T>
    where
        T: FromSql<Sqlite>
{
    fn from_sql(sql: &SqliteTypes) -> Result<Self, Error<Sqlite>>
        where
            Self: Sized
    {
        match sql {
            SqliteTypes::Null => Ok(None),
            _ => Ok(Some(T::from_sql(sql)?))
        }
    }
}

impl FromSql<Sqlite> for bool {
    fn from_sql(sql: &SqliteTypes) -> Result<Self, Error<Sqlite>>
        where
            Self: Sized
    {
        match sql {
            SqliteTypes::Integer(i) => Ok(*i != 0),
            _ => Err(Error::Conversion(format!("{:?}", sql), "bool"))
        }
    }
}

impl FromSql<Sqlite> for String {
    fn from_sql(sql: &SqliteTypes) -> Result<Self, Error<Sqlite>>
        where
            Self: Sized
    {
        match sql {
            SqliteTypes::Text(t) => Ok(t.clone()),
            _ => Err(Error::Conversion(format!("{:?}", sql), "String"))
        }
    }
}
