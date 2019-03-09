use crate::core::{backend::Database, table::Field, AsSql, QueryPart};
use std::fmt::Debug;

pub trait Condition<DB: Database>: QueryPart<DB> + Debug {
    fn and<Cond>(self, other: Cond) -> And<DB>
    where
        Self: Sized + 'static,
        Cond: Condition<DB> + 'static,
    {
        And::new(self, other)
    }
}

#[derive(Debug, Copy, Clone)]
pub struct EqField {
    pub field_1: Field,
    pub field_2: Field,
}

#[derive(Debug)]
pub struct EqValue<DB: Database> {
    pub field: Field,
    pub value: Box<dyn AsSql<DB>>,
}

#[derive(Debug)]
pub struct And<DB: Database> {
    pub cond_1: Box<dyn Condition<DB>>,
    pub cond_2: Box<dyn Condition<DB>>,
}

#[derive(Debug)]
pub struct Or<DB: Database> {
    pub cond_1: Box<dyn Condition<DB>>,
    pub cond_2: Box<dyn Condition<DB>>,
}

impl<'sql, DB: Database + 'sql> EqValue<DB> {
    pub fn new<S: AsSql<DB> + 'static>(field: Field, value: S) -> EqValue<DB> {
        EqValue {
            field,
            value: Box::new(value),
        }
    }
}

impl EqField {
    pub fn new(field_1: Field, field_2: Field) -> EqField {
        EqField { field_1, field_2 }
    }
}

impl<DB: Database> And<DB> {
    pub fn new<A: 'static, B: 'static>(cond_1: A, cond_2: B) -> And<DB>
    where
        A: Condition<DB>,
        B: Condition<DB>,
    {
        And {
            cond_1: Box::new(cond_1),
            cond_2: Box::new(cond_2),
        }
    }
}

impl<DB: Database> Or<DB> {
    pub fn new<A: 'static, B: 'static>(cond_1: A, cond_2: B) -> Or<DB>
    where
        A: Condition<DB>,
        B: Condition<DB>,
    {
        Or {
            cond_1: Box::new(cond_1),
            cond_2: Box::new(cond_2),
        }
    }
}

macro_rules! condition {
    ($cond_type: ty) => {
        impl<'a, DB: Database> Condition<DB> for $cond_type where $cond_type: QueryPart<DB> {}
    };
}

condition!(EqField);
condition!(And<DB>);
condition!(Or<DB>);
condition!(EqValue<DB>);
