use core::backend::Database;
use core::table::SetField;
use core::table::Table;

#[derive(Debug, Ord, PartialOrd, PartialEq, Eq, Copy, Clone)]
enum OnConflict {
    Fail,
    Ignore,
    Update,
}

#[derive(Debug)]
pub(crate) struct Insert<'a, DB: Database + 'a> {
    table: &'a Table,
    values: Vec<SetField<'a, DB>>,
    conflict: OnConflict,
}

impl<'a, DB: Database + 'a> Insert<'a, DB> {
    pub(crate) fn new(table: &'a Table, values: Vec<SetField<'a, DB>>) -> Insert<'a, DB> {
        Insert {
            table,
            values,
            conflict: OnConflict::Fail,
        }
    }

    pub(crate) fn values(&self) -> &Vec<SetField<'a, DB>> {
        &self.values
    }

    pub(crate) fn table(&self) -> &'a Table {
        self.table
    }

    pub(crate) fn with(mut self, value: SetField<'a, DB>) -> Insert<'a, DB> {
        self.values.push(value);
        self
    }

    pub(crate) fn on_conflict_update(mut self) -> Insert<'a, DB> {
        self.conflict = OnConflict::Update;
        self
    }

    pub(crate) fn on_conflict_ignore(mut self) -> Insert<'a, DB> {
        self.conflict = OnConflict::Ignore;
        self
    }
}

pub(crate) trait Insertable<DB: Database> {
    fn values(&self) -> Vec<SetField<DB>>;
    fn table<'a>(&'a self) -> &'a Table;

    fn insert<'a>(&'a self) -> Insert<'a, DB> {
        Insert {
            table: self.table(),
            values: self.values(),
            conflict: OnConflict::Fail,
        }
    }
}