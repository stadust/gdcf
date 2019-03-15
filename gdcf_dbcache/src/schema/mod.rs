#![allow(unused_imports)]

#[cfg(feature = "pg")]
use crate::core::backend::pg::Pg;
#[cfg(feature = "sqlite")]
use crate::core::backend::sqlite::Sqlite;
use crate::core::{
    backend::{Database, Error},
    query::select::{Queryable, Row},
    statement::{Preparation, Prepare},
    FromSql, QueryPart, SqlExpr,
};
use chrono::NaiveDateTime;
use gdcf::cache::CachedObject;

pub mod level;
pub mod song;
pub mod user;

impl<DB: Database, T: Queryable<DB>> Queryable<DB> for CachedObject<T>
where
    NaiveDateTime: FromSql<DB>,
{
    fn from_row(row: &Row<DB>, offset: isize) -> Result<Self, Error<DB>> {
        let t = T::from_row(row, offset)?;
        let first_cached = row.get(-2).unwrap()?;
        let lasted_cached = row.get(-1).unwrap()?;

        Ok(CachedObject::new(t, first_cached, lasted_cached))
    }
}

#[derive(Debug)]
pub struct NowAtUtc;

#[cfg(feature = "pg")]
impl QueryPart<Pg> for NowAtUtc {
    fn to_sql(&self) -> Preparation<Pg> {
        Preparation::<Pg>::default().with_static("(now() AT TIME ZONE 'utc')")
    }
}

#[cfg(feature = "pg")]
impl SqlExpr<Pg> for NowAtUtc {}

#[cfg(feature = "sqlite")]
impl QueryPart<Sqlite> for NowAtUtc {
    fn to_sql(&self) -> Preparation<Sqlite> {
        Preparation::<Sqlite>::default().with_static("(strftime('%Y-%m-%dT%H:%M:%S', CURRENT_TIMESTAMP))")
    }
}

#[cfg(feature = "sqlite")]
impl SqlExpr<Sqlite> for NowAtUtc {}
