use crate::core::{backend::Database, QueryPart};

pub trait Type<DB: Database>: QueryPart<DB> {}

#[derive(Debug, Default)]
pub struct UtcTimestamp;

#[derive(Debug, Default)]
pub struct SmallInteger;

#[derive(Debug, Default)]
pub struct Integer;

#[derive(Debug, Default)]
pub struct BigInteger;

#[derive(Debug, Default)]
pub struct Text;

#[derive(Debug, Default)]
pub struct Double;

#[derive(Debug, Default)]
pub struct Float;

#[derive(Debug, Default)]
pub struct Boolean;

#[derive(Debug, Default)]
pub struct Unsigned<Signed>(Signed);

#[derive(Debug, Default)]
pub struct Bytes;

if_query_part!(SmallInteger, Type<DB>);
if_query_part!(Integer, Type<DB>);
if_query_part!(BigInteger, Type<DB>);
if_query_part!(Text, Type<DB>);
if_query_part!(Double, Type<DB>);
if_query_part!(Float, Type<DB>);
if_query_part!(Boolean, Type<DB>);
if_query_part!(UtcTimestamp, Type<DB>);
if_query_part!(Bytes, Type<DB>);

impl<DB: Database, Signed> Type<DB> for Unsigned<Signed>
where
    Unsigned<Signed>: QueryPart<DB>,
    Signed: Type<DB>,
{
}
