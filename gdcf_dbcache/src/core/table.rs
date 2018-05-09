use core::query::condition::Condition;
use core::query::Select;
use super::{AsSql, backend::Database};
use super::query::condition::{EqField, EqValue};
use core::query::condition::And;

#[derive(Debug, PartialEq, Eq)]
pub(crate) struct Table {
    pub(crate) name: &'static str,
    pub(crate) fields: &'static [&'static Field],
}

impl Table {
    pub(crate) fn fields(&self) -> &'static [&'static Field] {
        &self.fields
    }

    pub(crate) fn filter<'a, DB, Cond: 'static>(&'a self, cond: Cond) -> Select<'a, DB>
        where
            Cond: Condition<'a, DB>,
            DB: Database,
            And<'a, DB>: Condition<'a, DB> + 'static,
    {
        Select::new(self, self.fields().to_vec())
            .filter(cond)
    }
}

#[derive(Debug, Eq, PartialEq)]
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

    pub(crate) fn set<'a, DB: Database + 'a>(&'a self, value: &'a AsSql<DB>) -> SetField<'a, DB> {
        SetField {
            field: &self,
            value: FieldValue::Value(value),
        }
    }

    pub(crate) fn set_default<DB: Database>(&self) -> SetField<DB> {
        SetField {
            field: &self,
            value: FieldValue::Default,
        }
    }

    pub fn name(&self) -> &str {
        self.name
    }

    pub fn qualified_name(&self) -> String {
        format!("{}.{}", self.table, self.name)
    }
}

#[derive(Debug)]
pub(crate) enum FieldValue<'a, DB: Database + 'a> {
    Default,
    Value(&'a AsSql<DB>),
}

impl<'a, DB: Database> FieldValue<'a, DB> {
    pub(crate) fn is_default(&self) -> bool {
        match self {
            FieldValue::Default => true,
            _ => false
        }
    }

    pub(crate) fn is_value(&self) -> bool {
        match self {
            FieldValue::Value(_) => true,
            _ => false
        }
    }
}

impl<'a, DB: Database, T: 'a> From<&'a T> for FieldValue<'a, DB>
    where
        T: AsSql<DB>
{
    fn from(t: &'a T) -> Self {
        FieldValue::Value(t)
    }
}

#[derive(Debug)]
pub(crate) struct SetField<'a, DB: Database + 'a> {
    pub(crate) field: &'a Field,
    pub(crate) value: FieldValue<'a, DB>,
}