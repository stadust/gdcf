use core::AsSql;
use core::backend::sqlite::Sqlite;
use core::backend::sqlite::SqliteTypes;
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

impl AsSql<Sqlite> for u8 {
    fn as_sql(&self) -> SqliteTypes {
        SqliteTypes::Integer(*self as i64)
    }
}

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