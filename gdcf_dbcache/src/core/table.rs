use super::{Database, AsSql};
use super::query::condition::{EqField, EqValue};

pub(crate) struct Table {
    pub(crate) name: &'static str,
    pub(crate) fields: &'static [&'static Field],
}

impl Table {
    pub(crate) fn fields(&self) -> &'static [&'static Field] {
        &self.fields
    }
}

pub(crate) struct Field {
    pub(crate) table: &'static str,
    pub(crate) name: &'static str,
}

impl Field {
    pub(crate) fn eq<'a, DB: Database + 'a>(&'a self, value: &'a AsSql<DB>) -> EqValue<'a, DB> {
        EqValue::new(&self, value)
    }

    pub(crate) fn same_as<'a>(&'a self, other: &'a Field) -> EqField<'a> {
        EqField::new(&self, other)
    }

    pub fn name(&self) -> &str {
        self.name
    }

    pub fn qualified_name(&self) -> String {
        format!("{}.{}", self.table, self.name)
    }
}