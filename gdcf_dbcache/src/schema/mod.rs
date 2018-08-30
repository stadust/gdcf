use chrono::NaiveDateTime;
use core::backend::Database;
use core::backend::Error;
#[cfg(feature = "pg")]
use core::backend::pg::Pg;
#[cfg(feature = "sqlite")]
use core::backend::sqlite::Sqlite;
use core::FromSql;
use core::query::select::Queryable;
use core::query::select::Row;
use core::QueryPart;
use core::SqlExpr;
use core::statement::Preparation;
use core::statement::Prepare;
use gdcf::cache::CachedObject;

pub mod song;
pub mod level;

impl<DB: Database, T: Queryable<DB>> Queryable<DB> for CachedObject<T>
    where
        NaiveDateTime: FromSql<DB>
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
        Preparation::<Pg>::default()
            .with_static("(now() AT TIME ZONE 'utc')")
    }
}

#[cfg(feature = "pg")]
impl SqlExpr<Pg> for NowAtUtc {}

#[cfg(feature = "sqlite")]
impl QueryPart<Sqlite> for NowAtUtc {
    fn to_sql(&self) -> Preparation<Sqlite> {
        Preparation::<Sqlite>::default()
            .with_static("(strftime('%Y-%m-%dT%H:%M:%S', CURRENT_TIMESTAMP))")
    }
}

#[cfg(feature = "sqlite")]
impl SqlExpr<Sqlite> for NowAtUtc {}