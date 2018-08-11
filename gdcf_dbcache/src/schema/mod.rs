use chrono::NaiveDateTime;
use core::backend::Database;
use core::backend::Error;
#[cfg(feature = "pg")]
use core::backend::pg::Pg;
use core::FromSql;
use core::query::QueryPart;
use core::query::select::Queryable;
use core::query::select::Row;
use core::SqlExpr;
use gdcf::cache::CachedObject;

pub mod song;
pub mod level;

impl<DB: Database, T: Queryable<DB>> Queryable<DB> for CachedObject<T>
    where
        NaiveDateTime: FromSql<DB>
{
    fn from_row(row: &Row<DB>, offset: isize) -> Result<Self, Error<DB>> {
        let t = T::from_row(row, offset)?;
        let first_cached = row.get(-1).unwrap()?;
        let lasted_cached = row.get(-2).unwrap()?;

        Ok(CachedObject::new(t, first_cached, lasted_cached))
    }
}

// TODO: find a fancier way to do this. For now it works though (maybe some sort of RawSqlExpr?)

#[derive(Debug)]
pub struct NowAtUtc;

#[cfg(feature = "pg")]
impl QueryPart<Pg> for NowAtUtc {
    fn to_sql_unprepared(&self) -> String {
        String::from("(now() AT TIME ZONE 'utc')")
    }
}

#[cfg(feature = "pg")]
impl SqlExpr<Pg> for NowAtUtc {}