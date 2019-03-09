use crate::core::{
    backend::{
        pg::{Pg, PgTypes},
        Database, Error,
    },
    AsSql, FromSql,
};
use chrono::NaiveDateTime;
use postgres::types::{FromSql as FromPgSql, IsNull, ToSql as ToPgSql, Type};
use std::{
    error::Error as StdError,
    fmt::{self, Display},
};

impl FromPgSql for PgTypes {
    fn from_sql(ty: &Type, raw: &[u8]) -> Result<Self, Box<dyn StdError + Send + Sync>> {
        if <bool as FromPgSql>::accepts(ty) {
            <bool as FromPgSql>::from_sql(ty, raw).map(PgTypes::Boolean)
        } else if <i16 as FromPgSql>::accepts(ty) {
            <i16 as FromPgSql>::from_sql(ty, raw).map(PgTypes::SmallInteger)
        } else if <i32 as FromPgSql>::accepts(ty) {
            <i32 as FromPgSql>::from_sql(ty, raw).map(PgTypes::Integer)
        } else if <i64 as FromPgSql>::accepts(ty) {
            <i64 as FromPgSql>::from_sql(ty, raw).map(PgTypes::BigInteger)
        } else if <String as FromPgSql>::accepts(ty) {
            <String as FromPgSql>::from_sql(ty, raw).map(PgTypes::Text)
        } else if <f32 as FromPgSql>::accepts(ty) {
            <f32 as FromPgSql>::from_sql(ty, raw).map(PgTypes::Float)
        } else if <f64 as FromPgSql>::accepts(ty) {
            <f64 as FromPgSql>::from_sql(ty, raw).map(PgTypes::Double)
        } else if <NaiveDateTime as FromPgSql>::accepts(ty) {
            <NaiveDateTime as FromPgSql>::from_sql(ty, raw).map(PgTypes::Timestamp)
        } else if <Vec<u8> as FromPgSql>::accepts(ty) {
            <Vec<u8> as FromPgSql>::from_sql(ty, raw).map(PgTypes::Bytes)
        } else {
            panic!("oh no!")
        }
    }

    fn from_sql_null(_: &Type) -> Result<Self, Box<dyn StdError + Send + Sync>> {
        Ok(PgTypes::Null)
    }

    fn accepts(ty: &Type) -> bool {
        <bool as FromPgSql>::accepts(ty)
            || <i16 as FromPgSql>::accepts(ty)
            || <i32 as FromPgSql>::accepts(ty)
            || <i64 as FromPgSql>::accepts(ty)
            || <String as FromPgSql>::accepts(ty)
            || <f32 as FromPgSql>::accepts(ty)
            || <f64 as FromPgSql>::accepts(ty)
            || <NaiveDateTime as FromPgSql>::accepts(ty)
            || <Vec<u8> as FromPgSql>::accepts(ty)
    }
}

impl ToPgSql for PgTypes {
    fn to_sql(&self, ty: &Type, out: &mut Vec<u8>) -> Result<IsNull, Box<dyn StdError + Send + Sync>>
    where
        Self: Sized,
    {
        match self {
            PgTypes::SmallInteger(value) => value.to_sql(ty, out),
            PgTypes::Integer(value) => value.to_sql(ty, out),
            PgTypes::Text(value) => value.to_sql(ty, out),
            PgTypes::BigInteger(value) => value.to_sql(ty, out),
            PgTypes::Double(value) => value.to_sql(ty, out),
            PgTypes::Float(value) => value.to_sql(ty, out),
            PgTypes::Boolean(value) => value.to_sql(ty, out),
            PgTypes::Timestamp(value) => value.to_sql(ty, out),
            PgTypes::Bytes(value) => value.to_sql(ty, out),
            PgTypes::Null => Ok(IsNull::Yes),
        }
    }

    fn accepts(_: &Type) -> bool
    where
        Self: Sized,
    {
        // Since in our to_sql_checked implementation we delegate to
        // the to_sql_checked implementation of other types, this method is never
        // called. Which is good, because we cannot possibly implement it
        // without a self reference, as we cannot statically know which enum
        // variant is used.
        true
    }

    fn to_sql_checked(&self, ty: &Type, out: &mut Vec<u8>) -> Result<IsNull, Box<dyn StdError + Send + Sync>> {
        match self {
            PgTypes::SmallInteger(value) => value.to_sql_checked(ty, out),
            PgTypes::Integer(value) => value.to_sql_checked(ty, out),
            PgTypes::Text(value) => value.to_sql_checked(ty, out),
            PgTypes::BigInteger(value) => value.to_sql_checked(ty, out),
            PgTypes::Double(value) => value.to_sql_checked(ty, out),
            PgTypes::Float(value) => value.to_sql_checked(ty, out),
            PgTypes::Boolean(value) => value.to_sql_checked(ty, out),
            PgTypes::Timestamp(value) => value.to_sql_checked(ty, out),
            PgTypes::Bytes(value) => value.to_sql_checked(ty, out),
            PgTypes::Null => Ok(IsNull::Yes),
        }
    }
}

impl Display for PgTypes {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        match self {
            PgTypes::SmallInteger(i) => write!(f, "{}", i),
            PgTypes::Integer(i) => write!(f, "{}", i),
            PgTypes::BigInteger(i) => write!(f, "{}", i),
            PgTypes::Text(t) => write!(f, "'{}'", t),
            PgTypes::Float(fl) => write!(f, "{}", fl),
            PgTypes::Double(d) => write!(f, "{}", d),
            PgTypes::Boolean(b) =>
                if *b {
                    write!(f, "TRUE")
                } else {
                    write!(f, "FALSE")
                },
            PgTypes::Timestamp(ts) => write!(f, "{}", ts.format("TIMESTAMP '%Y-%m-%d %H:%M:%S'")),
            PgTypes::Bytes(_) => write!(f, "<binary data>"), // TODO: figure this out
            PgTypes::Null => write!(f, "NULL"),
        }
    }
}

// Here we have impls that ensure that every AsSql<Sqlite> is also a
// SqlExpr<Sqlite> Maybe one day we'll find a better way to do this
mod _dummy {
    use super::*;
    use crate::core::{
        statement::{Preparation, PreparedStatement},
        QueryPart, SqlExpr,
    };

    impl<T: AsSql<Pg>> QueryPart<Pg> for T {
        fn to_sql(&self) -> Preparation<Pg> {
            (PreparedStatement::placeholder(), vec![self])
        }

        fn to_raw_sql(&self) -> String {
            self.as_sql().to_string()
        }
    }

    impl<'a> QueryPart<Pg> for dyn AsSql<Pg> + 'a {
        fn to_sql(&self) -> Preparation<Pg> {
            (PreparedStatement::placeholder(), vec![self])
        }

        fn to_raw_sql(&self) -> String {
            self.as_sql().to_string()
        }
    }

    impl<T: AsSql<Pg>> SqlExpr<Pg> for T {}
}

as_sql_cast_lossless!(Pg, i8, i16, PgTypes::SmallInteger);
as_sql_cast_lossless!(Pg, u8, i16, PgTypes::SmallInteger);
as_sql_cast!(Pg, i16, i16, PgTypes::SmallInteger);
as_sql_cast!(Pg, u16, i16, PgTypes::SmallInteger);
as_sql_cast!(Pg, i32, i32, PgTypes::Integer);
as_sql_cast!(Pg, u32, i32, PgTypes::Integer);
as_sql_cast!(Pg, i64, i64, PgTypes::BigInteger);
as_sql_cast!(Pg, u64, i64, PgTypes::BigInteger);

as_sql_cast!(Pg, bool, bool, PgTypes::Boolean);

as_sql_cast!(Pg, f32, f32, PgTypes::Float);
as_sql_cast!(Pg, f64, f64, PgTypes::Double);

impl AsSql<Pg> for String {
    fn as_sql(&self) -> PgTypes {
        PgTypes::Text(self.clone())
    }
}

impl<'a> AsSql<Pg> for &'a str {
    fn as_sql(&self) -> PgTypes {
        PgTypes::Text(self.to_string())
    }
}

impl AsSql<Pg> for NaiveDateTime {
    fn as_sql(&self) -> PgTypes {
        PgTypes::Timestamp(*self)
    }
}

impl<T> AsSql<Pg> for Option<T>
where
    T: AsSql<Pg>,
{
    fn as_sql(&self) -> PgTypes {
        match self {
            Some(value) => value.as_sql(),
            None => PgTypes::Null,
        }
    }
}

impl AsSql<Pg> for Vec<u8> {
    fn as_sql(&self) -> PgTypes {
        PgTypes::Bytes(self.clone())
    }
}

from_sql_cast!(Pg, u8, PgTypes::SmallInteger);
from_sql_cast!(Pg, i8, PgTypes::SmallInteger);
from_sql_cast!(Pg, i16, PgTypes::SmallInteger);
from_sql_cast!(Pg, u16, PgTypes::SmallInteger);
from_sql_cast!(Pg, i32, PgTypes::Integer);
from_sql_cast!(Pg, u32, PgTypes::Integer);
from_sql_cast!(Pg, i64, PgTypes::BigInteger);
from_sql_cast!(Pg, u64, PgTypes::BigInteger);

from_sql_cast!(Pg, f32, PgTypes::Float);
from_sql_cast!(Pg, f64, PgTypes::Double);

impl FromSql<Pg> for String {
    fn from_sql(sql: &PgTypes) -> Result<Self, Error<Pg>>
    where
        Self: Sized,
    {
        match sql {
            PgTypes::Text(value) => Ok(value.clone()),
            _ => Err(Error::Conversion(format!("{:?}", sql), "String")),
        }
    }
}

impl FromSql<Pg> for bool {
    fn from_sql(sql: &PgTypes) -> Result<Self, Error<Pg>>
    where
        Self: Sized,
    {
        match sql {
            PgTypes::Boolean(value) => Ok(*value),
            _ => Err(Error::Conversion(format!("{:?}", sql), "bool")),
        }
    }
}

impl<T> FromSql<Pg> for Option<T>
where
    T: FromSql<Pg>,
{
    fn from_sql(sql: &PgTypes) -> Result<Self, Error<Pg>>
    where
        Self: Sized,
    {
        match sql {
            PgTypes::Null => Ok(None),
            _ => T::from_sql(sql).map(Option::Some),
        }
    }
}

impl FromSql<Pg> for NaiveDateTime {
    fn from_sql(sql: &PgTypes) -> Result<Self, Error<Pg>>
    where
        Self: Sized,
    {
        match sql {
            PgTypes::Timestamp(ts) => Ok(*ts),
            _ => Err(Error::Conversion(format!("{:?}", sql), "NaiveDateTime")),
        }
    }
}

impl FromSql<Pg> for Vec<u8> {
    fn from_sql(sql: &PgTypes) -> Result<Self, Error<Pg>>
    where
        Self: Sized,
    {
        match sql {
            PgTypes::Bytes(vec) => Ok(vec.clone()),
            _ => Err(Error::Conversion(format!("{:?}", sql), "Vec<u8>")),
        }
    }
}
