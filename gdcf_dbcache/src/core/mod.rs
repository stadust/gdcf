use core::backend::Database;
use core::backend::Error;
use std::fmt::Debug;

//#[cfg(any(feature = "postgres", feature = "mysql", feature = "sqlite"))]
//pub mod cache;
//pub mod schema;
#[macro_use]
pub(crate) mod macros;
pub(crate) mod query;
pub(crate) mod table;
pub(crate) mod backend;
pub(crate) mod statement;
pub(crate) mod types;

pub(crate) trait AsSql<DB: Database>: Debug {
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

pub(crate) trait FromSql<DB: Database> {
    fn from_sql(sql: &DB::Types) -> Result<Self, Error<DB>>
        where
            Self: Sized;
}