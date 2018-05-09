use core::{AsSql, Database, table::Field};
use core::statement::PreparedStatement;
use super::QueryPart;

pub(crate) trait Condition<'a, DB: Database + 'a>: QueryPart<'a, DB> {
    fn and<Cond>(self, other: Cond) -> And<'a, DB>
        where
            Self: Sized + 'static,
            Cond: Condition<'a, DB> + 'static
    {
        And::new(self, other)
    }
}

#[derive(Debug, Copy, Clone)]
pub(crate) struct EqField<'a> {
    pub(crate) field_1: &'a Field,
    pub(crate) field_2: &'a Field,
}

#[derive(Copy, Clone)]
pub(crate) struct EqValue<'a, DB: Database + 'a> {
    pub(crate) field: &'a Field,
    pub(crate) value: &'a AsSql<DB>,
}

pub(crate) struct And<'a, DB: Database + 'a> {
    pub(crate) cond_1: Box<Condition<'a, DB>>,
    pub(crate) cond_2: Box<Condition<'a, DB>>,
}

pub(crate) struct Or<'a, DB: Database + 'a> {
    pub(crate) cond_1: Box<Condition<'a, DB>>,
    pub(crate) cond_2: Box<Condition<'a, DB>>,
}

impl<'a, DB: Database + 'a> EqValue<'a, DB> {
    pub(crate) fn new(field: &'a Field, value: &'a AsSql<DB>) -> EqValue<'a, DB> {
        EqValue {
            field,
            value,
        }
    }
}

impl<'a> EqField<'a> {
    pub(crate) fn new(field_1: &'a Field, field_2: &'a Field) -> EqField<'a> {
        EqField {
            field_1,
            field_2,
        }
    }
}

impl<'a, DB: Database + 'a> And<'a, DB> {
    pub(crate) fn new<A: 'static, B: 'static>(cond_1: A, cond_2: B) -> And<'a, DB>
        where
            A: Condition<'a, DB>,
            B: Condition<'a, DB>
    {
        And { cond_1: Box::new(cond_1), cond_2: Box::new(cond_2) }
    }
}

impl<'a, DB: Database + 'a> Or<'a, DB> {
    pub(crate) fn new<A: 'static, B: 'static>(cond_1: A, cond_2: B) -> Or<'a, DB>
        where
            A: Condition<'a, DB>,
            B: Condition<'a, DB>
    {
        Or { cond_1: Box::new(cond_1), cond_2: Box::new(cond_2) }
    }
}

macro_rules! condition {
    ($cond_type: ty) => {
        impl <'a, DB: Database + 'a> Condition<'a, DB> for $cond_type
            where
                $cond_type: QueryPart<'a, DB> {}
    };
}

condition!(EqField<'a>);
condition!(And<'a, DB>);
condition!(Or<'a, DB>);
condition!(EqValue<'a, DB>);