use chrono::NaiveDateTime;
use core::AsSql;
use core::backend::Error;
use core::backend::pg::Pg;
use core::FromSql;
use postgres::types::FromSql as FromPgSql;
use postgres::types::IsNull;
use postgres::types::ToSql as ToPgSql;
use postgres::types::Type;
use std::error::Error as StdError;
use chrono::format::StrftimeItems;

#[derive(Debug)]
pub(crate) enum PgTypes {
    SmallInteger(i16),
    Integer(i32),
    BigInteger(i64),
    Text(String),
    Float(f32),
    Double(f64),
    Boolean(bool),
    Timestamp(NaiveDateTime),
    Null,
}

impl FromPgSql for PgTypes {
    fn from_sql(ty: &Type, raw: &[u8]) -> Result<Self, Box<StdError + Send + Sync>> {
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
        } else {
            panic!("oh no!")
        }
    }

    fn accepts(ty: &Type) -> bool {
        <bool as FromPgSql>::accepts(ty) || <i16 as FromPgSql>::accepts(ty) ||
            <i32 as FromPgSql>::accepts(ty) || <i64 as FromPgSql>::accepts(ty) ||
            <String as FromPgSql>::accepts(ty) || <f32 as FromPgSql>::accepts(ty) ||
            <f64 as FromPgSql>::accepts(ty) || <NaiveDateTime as FromPgSql>::accepts(ty)
    }
}

impl ToPgSql for PgTypes {
    fn to_sql(&self, ty: &Type, out: &mut Vec<u8>) -> Result<IsNull, Box<StdError + Send + Sync>>
        where
            Self: Sized
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
            PgTypes::Null => Ok(IsNull::Yes)
        }
    }

    fn accepts(_: &Type) -> bool
        where
            Self: Sized
    {
        // Since in our to_sql_checked implementation we delegate to
        // the to_sql_checked implementation of other types, this method is never called.
        // Which is good, because we cannot possibly implement it without a self reference,
        // as we cannot statically know which enum variant is used.
        true
    }

    fn to_sql_checked(&self, ty: &Type, out: &mut Vec<u8>) -> Result<IsNull, Box<StdError + Send + Sync>> {
        match self {
            PgTypes::SmallInteger(value) => value.to_sql_checked(ty, out),
            PgTypes::Integer(value) => value.to_sql_checked(ty, out),
            PgTypes::Text(value) => value.to_sql_checked(ty, out),
            PgTypes::BigInteger(value) => value.to_sql_checked(ty, out),
            PgTypes::Double(value) => value.to_sql_checked(ty, out),
            PgTypes::Float(value) => value.to_sql_checked(ty, out),
            PgTypes::Boolean(value) => value.to_sql_checked(ty, out),
            PgTypes::Timestamp(value) => value.to_sql_checked(ty, out),
            PgTypes::Null => Ok(IsNull::Yes)
        }
    }
}

impl AsSql<Pg> for i8 {
    fn as_sql(&self) -> PgTypes {
        PgTypes::SmallInteger(*self as i16)
    }

    fn as_sql_string(&self) -> String {
        format!("{}", self)
    }
}

impl AsSql<Pg> for u8 {
    fn as_sql(&self) -> PgTypes {
        PgTypes::SmallInteger(*self as i16)
    }

    fn as_sql_string(&self) -> String {
        format!("{}", self)
    }
}

impl AsSql<Pg> for i16 {
    fn as_sql(&self) -> PgTypes {
        PgTypes::SmallInteger(*self)
    }

    fn as_sql_string(&self) -> String {
        format!("{}", self)
    }
}

impl AsSql<Pg> for u16 {
    fn as_sql(&self) -> PgTypes {
        PgTypes::SmallInteger(*self as i16)
    }

    fn as_sql_string(&self) -> String {
        format!("{}", self)
    }
}

impl AsSql<Pg> for i32 {
    fn as_sql(&self) -> PgTypes {
        PgTypes::Integer(*self)
    }

    fn as_sql_string(&self) -> String {
        format!("{}", self)
    }
}

impl AsSql<Pg> for u32 {
    fn as_sql(&self) -> PgTypes {
        PgTypes::Integer(*self as i32)
    }

    fn as_sql_string(&self) -> String {
        format!("{}", self)
    }
}

impl AsSql<Pg> for i64 {
    fn as_sql(&self) -> PgTypes {
        PgTypes::BigInteger(*self)
    }

    fn as_sql_string(&self) -> String {
        format!("{}", self)
    }
}

impl AsSql<Pg> for u64 {
    fn as_sql(&self) -> PgTypes {
        PgTypes::BigInteger(*self as i64)
    }

    fn as_sql_string(&self) -> String {
        format!("{}", self)
    }
}

impl AsSql<Pg> for f64 {
    fn as_sql(&self) -> PgTypes {
        PgTypes::Double(*self)
    }

    fn as_sql_string(&self) -> String {
        format!("{}", self)
    }
}

impl AsSql<Pg> for bool {
    fn as_sql(&self) -> PgTypes {
        PgTypes::Boolean(*self)
    }

    fn as_sql_string(&self) -> String {
        if *self {
            String::from("TRUE")
        } else {
            String::from("FALSE")
        }
    }
}

impl AsSql<Pg> for String {
    fn as_sql(&self) -> PgTypes {
        PgTypes::Text(self.clone())
    }

    fn as_sql_string(&self) -> String {
        self.clone()
    }
}

impl<'a> AsSql<Pg> for &'a str {
    fn as_sql(&self) -> PgTypes {
        PgTypes::Text(self.to_string())
    }

    fn as_sql_string(&self) -> String {
        format!("'{}'", self)
    }
}

impl AsSql<Pg> for NaiveDateTime {
    fn as_sql(&self) -> PgTypes {
        PgTypes::Timestamp(*self)
    }

    fn as_sql_string(&self) -> String {
        self.format("TIMESTAMP '%Y-%m-%d %H:%M:%S'").to_string()
    }
}

impl<T> AsSql<Pg> for Option<T>
    where
        T: AsSql<Pg>
{
    fn as_sql(&self) -> PgTypes {
        match self {
            Some(value) => value.as_sql(),
            None => PgTypes::Null
        }
    }

    fn as_sql_string(&self) -> String {
        match self {
            Some(value) => value.as_sql_string(),
            None => "NULL".to_string()
        }
    }
}

impl FromSql<Pg> for u8 {
    fn from_sql(sql: &PgTypes) -> Result<Self, Error<Pg>>
        where
            Self: Sized
    {
        match sql {
            PgTypes::SmallInteger(value) => Ok(*value as u8),
            _ => Err(Error::Conversion(format!("{:?}", sql), "u8"))
        }
    }
}

impl FromSql<Pg> for u64 {
    fn from_sql(sql: &PgTypes) -> Result<Self, Error<Pg>>
        where
            Self: Sized
    {
        match sql {
            PgTypes::BigInteger(value) => Ok(*value as u64),
            PgTypes::Integer(value) => Ok(*value as u64),
            _ => Err(Error::Conversion(format!("{:?}", sql), "u64"))
        }
    }
}

impl FromSql<Pg> for String {
    fn from_sql(sql: &PgTypes) -> Result<Self, Error<Pg>>
        where
            Self: Sized
    {
        match sql {
            PgTypes::Text(value) => Ok(value.clone()),
            _ => Err(Error::Conversion(format!("{:?}", sql), "String"))
        }
    }
}

impl FromSql<Pg> for i32 {
    fn from_sql(sql: &PgTypes) -> Result<Self, Error<Pg>>
        where
            Self: Sized
    {
        match sql {
            PgTypes::SmallInteger(value) => Ok((*value).into()),
            PgTypes::Integer(value) => Ok(*value),
            _ => Err(Error::Conversion(format!("{:?}", sql), "i32"))
        }
    }
}

impl FromSql<Pg> for u32 {
    fn from_sql(sql: &PgTypes) -> Result<Self, Error<Pg>>
        where
            Self: Sized
    {
        match sql {
            PgTypes::SmallInteger(value) => Ok(*value as u32), // do not sign extend here, it will fuck things up
            PgTypes::Integer(value) => Ok(*value as u32),
            _ => Err(Error::Conversion(format!("{:?}", sql), "u32"))
        }
    }
}

impl FromSql<Pg> for f64 {
    fn from_sql(sql: &PgTypes) -> Result<Self, Error<Pg>>
        where
            Self: Sized
    {
        match sql {
            PgTypes::Double(value) => Ok(*value),
            PgTypes::Float(value) => Ok(*value as f64),
            _ => Err(Error::Conversion(format!("{:?}", sql), "f64"))
        }
    }
}

impl FromSql<Pg> for bool {
    fn from_sql(sql: &PgTypes) -> Result<Self, Error<Pg>>
        where
            Self: Sized
    {
        match sql {
            PgTypes::Boolean(value) => Ok(*value),
            _ => Err(Error::Conversion(format!("{:?}", sql), "bool"))
        }
    }
}

impl<T> FromSql<Pg> for Option<T>
    where
        T: FromSql<Pg>
{
    fn from_sql(sql: &PgTypes) -> Result<Self, Error<Pg>>
        where
            Self: Sized
    {
        match sql {
            PgTypes::Null => Ok(None),
            _ => T::from_sql(sql).map(Option::Some)
        }
    }
}

impl FromSql<Pg> for NaiveDateTime {
    fn from_sql(sql: &PgTypes) -> Result<Self, Error<Pg>>
        where
            Self: Sized
    {
        match sql {
            PgTypes::Timestamp(ts) => Ok(*ts),
            _ => Err(Error::Conversion(format!("{:?}", sql), "NaiveDateTime"))
        }
    }
}