use core::{AsSql, backend::Database, table::Field};
use std::fmt::Debug;
use super::QueryPart;

pub trait Condition<DB: Database>: QueryPart<DB> + Debug {
    fn and<Cond>(self, other: Cond) -> And<DB>
        where
            Self: Sized + 'static,
            Cond: Condition<DB> + 'static
    {
        And::new(self, other)
    }
}

#[derive(Debug, Copy, Clone)]
pub struct EqField<'a> {
    pub field_1: &'a Field,
    pub field_2: &'a Field,
}

#[derive(Debug)]
pub struct EqValue<'a, DB: Database> {
    pub field: &'a Field,
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

impl<'f, 'sql, DB: Database + 'sql> EqValue<'f, DB> {
    pub fn new<S: AsSql<DB> + 'static>(field: &'f Field, value: S) -> EqValue<'f, DB> {
        EqValue {
            field,
            value: Box::new(value),
        }
    }
}

impl<'a> EqField<'a> {
    pub fn new(field_1: &'a Field, field_2: &'a Field) -> EqField<'a> {
        EqField {
            field_1,
            field_2,
        }
    }
}

impl<DB: Database> And<DB> {
    pub fn new<A: 'static, B: 'static>(cond_1: A, cond_2: B) -> And<DB>
        where
            A: Condition<DB>,
            B: Condition<DB>
    {
        And { cond_1: Box::new(cond_1), cond_2: Box::new(cond_2) }
    }
}

impl<DB: Database> Or<DB> {
    pub fn new<A: 'static, B: 'static>(cond_1: A, cond_2: B) -> Or<DB>
        where
            A: Condition<DB>,
            B: Condition<DB>
    {
        Or { cond_1: Box::new(cond_1), cond_2: Box::new(cond_2) }
    }
}

macro_rules! condition {
    ($cond_type: ty) => {
        impl<'a, DB: Database> Condition<DB> for $cond_type
            where
                $cond_type: QueryPart<DB> {}
    };
}

condition!(EqField<'a>);
condition!(And<DB>);
condition!(Or<DB>);
condition!(EqValue<'a, DB>);