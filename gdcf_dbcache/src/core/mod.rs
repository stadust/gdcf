use core::backend::Database;
use core::backend::Error;
use std::fmt::Debug;

//#[cfg(any(feature = "postgres", feature = "mysql", feature = "sqlite"))]
//pub mod cache;
//pub mod schema;
#[macro_use]
pub  mod macros;
pub  mod query;
pub  mod table;
pub  mod backend;
pub  mod statement;
pub  mod types;

pub trait AsSql<DB: Database>: Debug {
    fn as_sql(&self) -> DB::Types;
    fn as_sql_string(&self) -> String;
}

impl<'a, T: 'a, DB: Database + 'a> AsSql<DB> for &'a T
    where
        T: AsSql<DB>
{
    fn as_sql(&self) -> <DB as Database>::Types {
        (*self).as_sql()
    }

    fn as_sql_string(&self) -> String {
        (*self).as_sql_string()
    }
}

pub  trait FromSql<DB: Database> {
    fn from_sql(sql: &DB::Types) -> Result<Self, Error<DB>>
        where
            Self: Sized;
}