use core::backend::Database;
use core::query::Insertable;
use core::table::SetField;
use core::table::Table;
use gdcf::cache::CachedObject;
use core::query::select::Queryable;
use core::query::select::Row;
use core::backend::Error;

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

impl<DB: Database, T: Queryable<DB>> Queryable<DB> for CachedObject<T> {
    fn from_row(row: &Row<DB>, offset: isize) -> Result<Self, Error<DB>> {
        let t = T::from_row(row, offset)?;

        // TODO: get cache data from row.get(-1) and row.get(-2) and construct CachedObject<T>
        unimplemented!()
    }
}