use std::fmt::Debug;
use core::backend::Database;

//#[cfg(any(feature = "postgres", feature = "mysql", feature = "sqlite"))]
//pub mod cache;
//pub mod schema;
#[macro_use]
pub(crate) mod macros;
pub(crate) mod query;
pub(crate) mod table;
pub(crate) mod backend;
pub(crate) mod statement;

pub(crate) trait AsSql<DB: Database>: Debug {
    fn as_sql(&self) -> DB::Types;
    fn as_sql_string(&self) -> String;
}

pub(crate) trait FromSql<DB: Database> {
    fn from_sql(sql: DB::Types) -> Self;
}