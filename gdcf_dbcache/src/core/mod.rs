use core::backend::Database;
use core::backend::Error;
use core::statement::Preparation;
use core::statement::Prepare;
//use std::fmt::{self, Display};
use std::fmt::Debug;

#[macro_use]
pub mod macros;
pub mod query;
pub mod table;
pub mod backend;
pub mod statement;
pub mod types;
/*
pub trait AsRawSql<DB: Database> {
    fn as_raw_sql(&self) -> String;
}*/

pub trait AsSql<DB: Database>: Debug {
    fn as_sql(&self) -> DB::Types;
}

impl<'a, T: AsSql<DB> + 'a, DB: Database> AsSql<DB> for &'a T {
    fn as_sql(&self) -> <DB as Database>::Types {
        (*self).as_sql()
    }
}
/*
// TODO: what the fuck
impl<DB: Database> AsSql<DB> for Box<dyn AsSql<DB>> {
    fn as_sql(&self) -> <DB as Database>::Types {
        (*self).as_sql()
    }
}
*/
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
/*
impl<DB: Database, T: AsSql<DB>> QueryPart<DB> for T {
    fn to_sql(&self) -> Preparation<DB> {
        (PreparedStatement::placeholder(), vec!(self))
    }
}

impl<'a, DB: Database> QueryPart<DB> for dyn AsSql<DB> + 'a {
    fn to_sql(&self) -> Preparation<DB> {
        (PreparedStatement::placeholder(), vec!(self))
    }

    fn to_raw_sql(&self) -> String {
        self.as_sql().to_string()
    }
}*/

// Alright, so the optimal solution here would be
//  SqlExpr<DB>: QueryPart<DB>
//  impl<DB, T> QueryPart<DB> for T where T: AsSql<DB>
//  impl<DB, T> SqlExpr<DB> for T where T: AsSql<DB>
// but due to the "downstream crate may implement QueryPart<DB> for _" bullshit
// we cannot do that. Which is why SqlExpr<DB> is a duplicate of
pub trait SqlExpr<DB: Database>: QueryPart<DB> {}

#[derive(Debug)]
pub struct RawSql(pub &'static str);

impl<DB: Database> QueryPart<DB> for RawSql {
    fn to_sql(&self) -> Preparation<DB> {
        Preparation::<DB>::default()
            .with_static(self.0)
    }
}

impl<DB: Database> SqlExpr<DB> for RawSql {}
/*impl Display for RawSql {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        write!(f, "{}", self.0)
    }
}

impl<DB: Database> SqlExpr<DB> for RawSql{}*/