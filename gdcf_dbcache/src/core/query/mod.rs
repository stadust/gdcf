use core::{AsSql, backend::Database, FromSql, statement::PreparedStatement};
pub(crate) use self::insert::{Insert, Insertable};
pub(crate) use self::select::{Join, Select};

pub(crate) mod condition;
pub(crate) mod insert;
pub(crate) mod select;

pub(crate) trait QueryPart<'a, DB: Database + 'a> {
    fn to_sql_unprepared(&self) -> String;

    fn to_sql(&self) -> (PreparedStatement, Vec<&'a AsSql<DB>>) {
        (self.to_sql_unprepared().into(), Vec::new())
    }
}

pub(crate) trait Query<'a, DB: Database + 'a>: QueryPart<'a, DB> {
    fn execute(&'a self, db: &'a DB) -> Result<(), DB::Error>
        where
            Self: Sized
    {
        db.execute(self)
    }

    fn execute_unprepared(&'a self, db: &'a DB) -> Result<(), DB::Error>
        where
            Self: Sized
    {
        db.execute_unprepared(self)
    }
}

trait Row<DB: Database> {
    fn get<T>(idx: usize) -> T
        where
            T: FromSql<DB>;
}