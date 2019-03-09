use crate::core::{
    backend::Database,
    query::{
        condition::{And, Condition, EqField, EqValue},
        create::Create,
        Select,
    },
    AsSql,
};

#[derive(Debug, PartialEq, Eq, Copy, Clone)]
pub struct Table {
    pub name: &'static str,
    pub fields: &'static [Field],
}

impl Table {
    pub fn fields(&self) -> &'static [Field] {
        &self.fields
    }

    pub fn filter<DB, Cond>(&self, cond: Cond) -> Select<DB>
    where
        Cond: Condition<DB> + 'static,
        DB: Database,
        And<DB>: Condition<DB> + 'static,
    {
        self.select().filter(cond)
    }

    pub fn select<DB: Database>(self) -> Select<DB> {
        Select::new(self, self.fields().to_vec())
    }

    pub fn create<DB>(&self) -> Create<DB>
    where
        DB: Database,
    {
        Create::new(self.name)
    }
}

#[derive(Debug, Eq, PartialEq, Copy, Clone)]
pub struct Field {
    pub table: &'static str,
    pub name: &'static str,
}

impl Field {
    pub fn eq<DB: Database, S: AsSql<DB> + 'static>(self, value: S) -> EqValue<DB> {
        EqValue::new(self, value)
    }

    pub fn same_as(self, other: Field) -> EqField {
        EqField::new(self, other)
    }

    pub fn set<'a, DB: Database + 'a>(self, value: &'a dyn AsSql<DB>) -> SetField<'a, DB> {
        SetField {
            field: self,
            value: FieldValue::Value(value),
        }
    }

    pub fn set_default<DB: Database>(self) -> SetField<'static, DB> {
        SetField {
            field: self,
            value: FieldValue::Default,
        }
    }

    pub fn name(self) -> &'static str {
        self.name
    }

    pub fn qualified_name(self) -> String {
        format!("{}.{}", self.table, self.name)
    }
}

#[derive(Debug)]
pub enum FieldValue<'a, DB: Database + 'a> {
    Default,
    Value(&'a dyn AsSql<DB>),
}

impl<'a, DB: Database> FieldValue<'a, DB> {
    pub fn is_default(&self) -> bool {
        match self {
            FieldValue::Default => true,
            _ => false,
        }
    }

    pub fn is_value(&self) -> bool {
        match self {
            FieldValue::Value(_) => true,
            _ => false,
        }
    }
}

impl<'a, DB: Database, T: 'a> From<&'a T> for FieldValue<'a, DB>
where
    T: AsSql<DB>,
{
    fn from(t: &'a T) -> Self {
        FieldValue::Value(t)
    }
}

#[derive(Debug)]
pub struct SetField<'a, DB: Database + 'a> {
    pub field: Field,
    pub value: FieldValue<'a, DB>,
}
