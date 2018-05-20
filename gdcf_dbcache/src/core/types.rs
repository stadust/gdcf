use core::backend::Database;
use core::query::QueryPart;

pub(crate) trait Type<'a, DB: Database + 'a>: QueryPart<'a, DB> {}

#[derive(Debug, Default)]
pub(crate) struct UtcTimestamp;

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

if_query_part!(Integer, Type<'a, DB>);
if_query_part!(BigInteger, Type<'a, DB>);
if_query_part!(Text, Type<'a, DB>);
if_query_part!(Double, Type<'a, DB>);
if_query_part!(Float, Type<'a, DB>);
if_query_part!(Boolean, Type<'a, DB>);
if_query_part!(UtcTimestamp, Type<'a, DB>);

impl<'a, DB: Database + 'a, Signed> Type<'a, DB> for Unsigned<Signed>
    where
        Unsigned<Signed>: QueryPart<'a, DB>,
        Signed: Type<'a, DB> {}