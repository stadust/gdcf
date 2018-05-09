use core::backend::pg::Pg;
use core::AsSql;
use postgres::types::IsNull;
use postgres::types::Type;
use postgres::types::ToSql as ToPgSql;
use std::error::Error;

#[derive(Debug)]
pub(crate) enum PgTypes {
    Integer(i32),
    Text(String)
}

impl ToPgSql for PgTypes {
    fn to_sql(&self, ty: &Type, out: &mut Vec<u8>) -> Result<IsNull, Box<Error + Send + Sync>>
        where
            Self: Sized
    {
        match self {
            PgTypes::Integer(value) => value.to_sql(ty, out),
            PgTypes::Text(value) => value.to_sql(ty, out)
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

    fn to_sql_checked(&self, ty: &Type, out: &mut Vec<u8>) -> Result<IsNull, Box<Error + Send + Sync>> {
        match self {
            PgTypes::Integer(value) => value.to_sql_checked(ty, out),
            PgTypes::Text(value) => value.to_sql_checked(ty, out),
        }
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