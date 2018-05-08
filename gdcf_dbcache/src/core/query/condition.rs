use core::{Database, AsSql, table::Field};
use super::QueryPart;

pub(crate) trait Condition<'a, DB: Database + 'a>: QueryPart<'a, DB> {
    fn and(&'a self, other: &'a Condition<'a, DB>) -> And<'a, DB>
        where
            Self: Sized {
        And {
            cond_1: self,
            cond_2: other,
        }
    }
}

pub(crate) struct EqField<'a> {
    pub(crate) field_1: &'a Field,
    pub(crate) field_2: &'a Field,
}

pub(crate) struct EqValue<'a, DB: Database + 'a> {
    pub(crate) field: &'a Field,
    pub(crate) value: &'a AsSql<DB>,
}

pub(crate) struct And<'a, DB: Database + 'a> {
    pub(crate) cond_1: &'a Condition<'a, DB>,
    pub(crate) cond_2: &'a Condition<'a, DB>,
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

macro_rules! condition {
    ($cond_type: ty) => {
        impl <'a, DB: Database + 'a> Condition<'a, DB> for $cond_type
            where
                $cond_type: QueryPart<'a,   DB>{}
    };
}

condition!(EqField<'a>);
condition!(And<'a, DB>);
condition!(EqValue<'a, DB>);