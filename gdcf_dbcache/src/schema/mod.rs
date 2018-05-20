use chrono::NaiveDateTime;
use core::backend::Database;
use core::backend::Error;
use core::FromSql;
use core::query::Insertable;
use core::query::select::Queryable;
use core::query::select::Row;
use core::table::SetField;
use core::table::Table;
use gdcf::cache::CachedObject;

pub mod song;
pub mod level;

impl<DB: Database, T: Insertable<DB>> Insertable<DB> for CachedObject<T> {
    fn values(&self) -> Vec<SetField<DB>> {
        let values = self.cached().values();

        // TODO: deal with the timestamp types here

        values
    }

    fn table<'a>(&'a self) -> &Table {
        self.cached().table()
    }
}

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