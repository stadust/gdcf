use core::Database;
use core::AsSql;

struct Sqlite;


impl Database for Sqlite {
    type Types = ();
    type Error = ();

    fn prepare(idx: usize) -> String {
        unimplemented!()
    }

    fn execute_raw(&self, statement: String, params: &[&AsSql<Self>]) -> Result<(), <Self as Database>::Error> {
        unimplemented!()
    }
}