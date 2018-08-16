use core::AsSql;
use core::backend::Database;
use core::query::condition::And;
use core::QueryPart;
use core::query::select::Row;
use core::statement::{PreparedStatement, StatementPart};
use core::query::condition::EqValue;

#[derive(Debug)]
pub struct Sqlite;


impl Database for Sqlite {
    type Types = ();
    type Error = ();

    fn prepare(idx: usize) -> String {
        unimplemented!()
    }

    fn execute_raw(&self, statement: String, params: &[&AsSql<Self>]) -> Result<(), <Self as Database>::Error> {
        unimplemented!()
    }

    fn query_raw(&self, statement: String, params: &[&AsSql<Self>]) -> Result<Vec<Row<Self>>, <Self as Database>::Error> where
        Self: Sized {
        unimplemented!()
    }
}

impl QueryPart<Sqlite> for And<Sqlite> {
    fn to_sql_unprepared(&self) -> String {
        format!("({} AND {})", self.cond_1.to_sql_unprepared(), self.cond_2.to_sql_unprepared())
    }

    fn to_sql(&self) -> (PreparedStatement, Vec<&'a AsSql<Sqlite>>) {
        let (mut stmt1, mut params1) = self.cond_1.to_sql();
        let (mut stmt2, mut params2) = self.cond_1.to_sql();

        params1.append(&mut params2);

        stmt1.prepend("(");
        stmt2.append(")");

        stmt1.concat_on(stmt2, " AND ");

        (stmt1, params1)
    }
}

impl AsSql<Sqlite> for i32 {
    fn as_sql(&self) -> () {
        ()
    }

    fn as_sql_string(&self) -> String {
        format!("{}", self)
    }
}

impl<'a> QueryPart<'a, Sqlite> for EqValue<'a, Sqlite> {
    fn to_sql_unprepared(&self) -> String {
        format!("({} = {})", self.field.qualified_name(), self.value.as_sql_string())
    }

    fn to_sql(&self) -> (PreparedStatement, Vec<&'a AsSql<Sqlite>>) {
        let stmt = PreparedStatement::new(vec![
            format!("({} = ", self.field.qualified_name()).into(),
            StatementPart::Placeholder,
            ")".into()
        ]);

        (stmt, vec![self.value])
    }
}