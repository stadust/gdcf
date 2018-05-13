use core::backend::Database;
use core::query::Insertable;
use core::table::SetField;
use core::table::Table;
use gdcf::cache::CachedObject;

pub mod song;


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