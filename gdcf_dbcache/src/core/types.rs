use core::backend::Database;
use core::query::QueryPart;
use std::marker::PhantomData;

pub(crate) trait Type<'a, DB: Database + 'a>: QueryPart<'a, DB> {}

#[derive(Debug, Default)]
pub(crate) struct Timestamp;

#[derive(Debug, Default)]
pub(crate) struct TinyInteger;

#[derive(Debug, Default)]
pub(crate) struct SmallInteger;

#[derive(Debug, Default)]
pub(crate) struct Integer;

#[derive(Debug, Default)]
pub(crate) struct BigInteger;

#[derive(Debug, Default)]
pub(crate) struct Text;

#[derive(Debug, Default)]
pub(crate) struct Double;

#[derive(Debug, Default)]
pub(crate) struct Float;

#[derive(Debug, Default)]
pub(crate) struct Boolean;

#[derive(Debug, Default)]
pub(crate) struct Unsigned<Signed>(Signed);

macro_rules! implement {
    ($t: ty) => {
        impl<'a, DB: Database + 'a> Type<'a, DB> for $t
            where
                $t: QueryPart<'a, DB> {}
    };
}

implement!(Integer);
implement!(BigInteger);
implement!(Text);
implement!(Double);
implement!(Float);
implement!(Boolean);

impl<'a, DB: Database + 'a, Signed> Type<'a, DB> for Unsigned<Signed>
    where
        Unsigned<Signed>: QueryPart<'a, DB>,
        Signed: Type<'a, DB> {}