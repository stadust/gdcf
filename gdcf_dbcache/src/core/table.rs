use core::query::condition::And;
use core::query::condition::Condition;
use core::query::create::Create;
use core::query::Select;
use super::{AsSql, backend::Database};
use super::query::condition::{EqField, EqValue};

#[derive(Debug, PartialEq, Eq)]
pub struct Table {
    pub name: &'static str,
    pub fields: &'static [&'static Field],
}

impl Table {
    pub fn fields(&self) -> &'static [&'static Field] {
        &self.fields
    }

    pub fn filter<DB, Cond>(&self, cond: Cond) -> Select<DB>
        where
            Cond: Condition<DB> + 'static,
            DB: Database,
     //       And<DB>: Condition<DB> + 'static,
    {
        self.select().filter(cond)
    }

    pub fn select<DB: Database>(&self) -> Select<DB> {
        Select::new(self, self.fields().to_vec())
    }

    pub fn create<DB>(&self) -> Create<DB>
        where
            DB: Database
    {
        Create::new(self.name)
    }
}

#[derive(Debug, Eq, PartialEq)]
pub struct Field {
    pub table: &'static str,
    pub name: &'static str,
}

impl Field {
    pub fn eq<'f, DB: Database, S: AsSql<DB> + 'static>(&'f self, value: S) -> EqValue<'f, DB> {
        EqValue::new(&self, value)
    }

    pub fn same_as<'a>(&'a self, other: &'a Field) -> EqField<'a> {
        EqField::new(&self, other)
    }

    pub fn set<'a, DB: Database + 'a>(&'a self, value: &'a AsSql<DB>) -> SetField<'a, DB> {
        SetField {
            field: &self,
            value: FieldValue::Value(value),
        }
    }

    pub fn set_default<DB: Database>(&self) -> SetField<DB> {
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
pub enum FieldValue<'a, DB: Database + 'a> {
    Default,
    Value(&'a AsSql<DB>),
}

impl<'a, DB: Database> FieldValue<'a, DB> {
    pub fn is_default(&self) -> bool {
        match self {
            FieldValue::Default => true,
            _ => false
        }
    }

    pub fn is_value(&self) -> bool {
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
pub struct SetField<'a, DB: Database + 'a> {
    pub field: &'a Field,
    pub value: FieldValue<'a, DB>,
}