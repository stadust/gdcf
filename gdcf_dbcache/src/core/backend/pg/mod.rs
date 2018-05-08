use core::{AsSql, Database, query::InsertValue};
use core::query::Insertable;
use postgres::{Connection, Error, types::ToSql as ToPgSql};
use postgres::types::IsNull;
use postgres::types::Type;
use self::types::PgTypes;
use std::error;

mod condition;
mod types;
mod query;

struct Pg {
    conn: Connection
}

impl Database for Pg {
    type Types = PgTypes;
    type Error = Error;

    fn prepare(idx: usize) -> String {
        format!("${}", idx)
    }

    fn execute_raw(&self, statement: String, params: &[&AsSql<Self>]) -> Result<(), <Self as Database>::Error> {
        let comp: Vec<_> = params.into_iter().map(|param| param.as_sql()).collect();
        let values: Vec<_> = comp.iter().map(|v| v as &ToPgSql).collect();

        self.conn.execute(&statement, &values[..])?;
        Ok(())
    }
}

struct Test;

impl Insertable<Pg> for Test {
    fn values<'a>(&'a self) -> Vec<InsertValue<'a, Pg>> {
        vec![InsertValue::Default, (&6i32).into()]
    }
}