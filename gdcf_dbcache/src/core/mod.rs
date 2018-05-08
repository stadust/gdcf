use self::query::Query;

//#[cfg(any(feature = "postgres", feature = "mysql", feature = "sqlite"))]
//pub mod cache;
//pub mod schema;
#[macro_use]
mod macros;
mod query;
mod table;
mod backend;
mod statement;

pub(crate) trait AsSql<DB: Database> {
    fn as_sql(&self) -> DB::Types;
    fn as_sql_string(&self) -> String;
}

pub(crate) trait FromSql<DB: Database> {
    fn from_sql(sql: DB::Types) -> Self;
}

pub(crate) trait Database {
    type Types;
    type Error;

    fn prepare(idx: usize) -> String;

    fn execute<'a>(&'a self, query: &'a Query<'a, Self>) -> Result<(), Self::Error>
        where
            Self: Sized
    {
        let (stmt, params) = query.to_sql();
        self.execute_raw(stmt.to_statement(Self::prepare), &params)
    }

    fn execute_unprepared<'a>(&'a self, query: &'a Query<'a, Self>) -> Result<(), Self::Error>
        where
            Self: Sized
    {
        self.execute_raw(query.to_sql_unprepared(), &[])
    }

    fn execute_raw(&self, statement: String, params: &[&AsSql<Self>]) -> Result<(), Self::Error>;
}

table! {
    song => {
        name
    }
}