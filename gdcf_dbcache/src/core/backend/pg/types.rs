use core::backend::pg::Pg;
use core::query::QueryPart;
use core::types::{BigInteger, Boolean, Double, Float, Integer, Text, Unsigned};

macro_rules! impl_type {
    ($t: ty, $val: expr) => {
        impl<'a> QueryPart<'a, Pg> for $t {
            fn to_sql_unprepared(&self) -> String {
                String::from($val)
            }
        }
    };
}

impl_type!(Text, "TEXT");
impl_type!(Integer, "INT");
impl_type!(BigInteger, "BIGINT");
impl_type!(Boolean, "BOOL");
impl_type!(Float, "FLOAT(4)");
impl_type!(Double, "REAL");
impl_type!(Unsigned<Integer>, "INTEGER");
impl_type!(Unsigned<BigInteger>, "BIGINT");