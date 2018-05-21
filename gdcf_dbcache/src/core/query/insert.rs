use core::backend::Database;
use core::query::Query;
use core::query::QueryPart;
use core::table::SetField;
use core::table::Table;

#[derive(Debug, Ord, PartialOrd, PartialEq, Eq, Copy, Clone)]
enum OnConflict {
    Fail,
    Ignore,
    Update,
}

#[derive(Debug)]
pub struct Insert<'a, DB: Database + 'a> {
    table: &'a Table,
    values: Vec<SetField<'a, DB>>,
    conflict: OnConflict,
}

impl<'a, DB: Database + 'a> Insert<'a, DB> {
    pub fn new(table: &'a Table, values: Vec<SetField<'a, DB>>) -> Insert<'a, DB> {
        Insert {
            table,
            values,
            conflict: OnConflict::Fail,
        }
    }

    pub fn values(&self) -> &Vec<SetField<'a, DB>> {
        &self.values
    }

    pub fn table(&self) -> &'a Table {
        self.table
    }

    pub fn with(mut self, value: SetField<'a, DB>) -> Insert<'a, DB> {
        self.values.push(value);
        self
    }

    pub fn on_conflict_update(mut self) -> Insert<'a, DB> {
        self.conflict = OnConflict::Update;
        self
    }

    pub fn on_conflict_ignore(mut self) -> Insert<'a, DB> {
        self.conflict = OnConflict::Ignore;
        self
    }
}

pub trait Insertable<DB: Database> {
    fn values(&self) -> Vec<SetField<DB>>;
    fn table(&self) -> &Table;

    fn insert(&self) -> Insert<DB> {
        Insert {
            table: self.table(),
            values: self.values(),
            conflict: OnConflict::Fail,
        }
    }
}

impl<'a, DB: Database + 'a> Query<'a, DB> for Insert<'a, DB>
    where
        Insert<'a, DB>: QueryPart<'a, DB> {}