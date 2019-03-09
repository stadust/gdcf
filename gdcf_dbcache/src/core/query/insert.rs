use crate::core::{
    backend::Database,
    query::Query,
    table::{Field, SetField, Table},
    QueryPart,
};

#[derive(Debug, PartialEq, Eq, Clone)]
pub(crate) enum OnConflict {
    Fail,
    Ignore,
    Update(Vec<&'static Field>),
}

#[derive(Debug)]
pub struct Insert<'a, DB: Database + 'a> {
    table: Table,
    values: Vec<SetField<'a, DB>>,
    // TODO: multiple rows in one insert
    pub(crate) conflict: OnConflict,
}

impl<'a, DB: Database + 'a> Insert<'a, DB> {
    pub fn new(table: Table, values: Vec<SetField<'a, DB>>) -> Insert<'a, DB> {
        Insert {
            table,
            values,
            conflict: OnConflict::Fail,
        }
    }

    pub fn values(&self) -> &Vec<SetField<'a, DB>> {
        &self.values
    }

    pub fn table(&self) -> Table {
        self.table
    }

    pub fn with(mut self, value: SetField<'a, DB>) -> Insert<'a, DB> {
        self.values.push(value);
        self
    }

    pub fn on_conflict_update(mut self, fields: Vec<&'static Field>) -> Insert<'a, DB> {
        self.conflict = OnConflict::Update(fields);
        self
    }

    pub fn on_conflict_ignore(mut self) -> Insert<'a, DB> {
        self.conflict = OnConflict::Ignore;
        self
    }
}

pub trait Insertable<DB: Database> {
    fn values(&self) -> Vec<SetField<DB>>;
    fn table(&self) -> Table;

    fn insert(&self) -> Insert<DB> {
        Insert {
            table: self.table(),
            values: self.values(),
            conflict: OnConflict::Fail,
        }
    }
}

if_query_part!(Insert<'a, DB>, Query<DB>);
