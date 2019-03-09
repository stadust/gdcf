use crate::core::{
    backend::{Database, Error},
    statement::{Preparation, Prepare},
};
use std::fmt::Debug;

#[macro_use]
pub mod macros;
pub mod backend;
pub mod query;
pub mod statement;
pub mod table;
pub mod types;

pub trait AsSql<DB: Database>: Debug {
    fn as_sql(&self) -> DB::Types;
}

impl<'a, T: AsSql<DB> + 'a, DB: Database> AsSql<DB> for &'a T {
    fn as_sql(&self) -> <DB as Database>::Types {
        (*self).as_sql()
    }
}

pub trait FromSql<DB: Database> {
    fn from_sql(sql: &DB::Types) -> Result<Self, Error<DB>>
    where
        Self: Sized;
}

pub trait QueryPart<DB: Database>: Debug {
    fn to_sql(&self) -> Preparation<DB>;
    fn to_raw_sql(&self) -> String {
        self.to_sql().unprepared()
    }
}

// Alright, so the optimal solution here would be
//  SqlExpr<DB>: QueryPart<DB>
//  impl<DB, T> QueryPart<DB> for T where T: AsSql<DB>
//  impl<DB, T> SqlExpr<DB> for T where T: AsSql<DB>
// but due to the "downstream crate may implement QueryPart<DB> for _" bullshit
// we cannot do that. So every backend needs to do the above two impl
// specialized to itself.
pub trait SqlExpr<DB: Database>: QueryPart<DB> {}
