use core::{AsSql, query::{condition::{And, EqField, EqValue, Or}, QueryPart}, statement::{PreparedStatement, StatementPart}};
use super::Pg;

impl<'a> QueryPart<'a, Pg> for EqField<'a> {
    fn to_sql_unprepared(&self) -> String {
        format!("({} = {})", self.field_1.qualified_name(), self.field_2.qualified_name())
    }
}

impl<'a> QueryPart<'a, Pg> for EqValue<'a, Pg> {
    fn to_sql_unprepared(&self) -> String {
        format!("({} = {})", self.field.qualified_name(), self.value.as_sql_string())
    }

    fn to_sql(&self) -> (PreparedStatement, Vec<&'a AsSql<Pg>>) {
        let stmt = PreparedStatement::new(vec![
            format!("({} = ", self.field.qualified_name()).into(),
            StatementPart::Placeholder,
            ")".into()
        ]);

        (stmt, vec![self.value])
    }
}

impl<'a> QueryPart<'a, Pg> for And<'a, Pg> {
    fn to_sql_unprepared(&self) -> String {
        format!("({} AND {})", self.cond_1.to_sql_unprepared(), self.cond_2.to_sql_unprepared())
    }

    fn to_sql(&self) -> (PreparedStatement, Vec<&'a AsSql<Pg>>) {
        let (mut stmt1, mut params1) = self.cond_1.to_sql();
        let (mut stmt2, mut params2) = self.cond_1.to_sql();

        params1.append(&mut params2);

        stmt1.prepend("(");
        stmt2.append(")");

        stmt1.concat_on(stmt2, " AND ");

        (stmt1, params1)
    }
}


impl<'a> QueryPart<'a, Pg> for Or<'a, Pg> {
    fn to_sql_unprepared(&self) -> String {
        format!("({} OR {})", self.cond_1.to_sql_unprepared(), self.cond_2.to_sql_unprepared())
    }

    fn to_sql(&self) -> (PreparedStatement, Vec<&'a AsSql<Pg>>) {
        let (mut stmt1, mut params1) = self.cond_1.to_sql();
        let (mut stmt2, mut params2) = self.cond_1.to_sql();

        params1.append(&mut params2);

        stmt1.prepend("(");
        stmt2.append(")");

        stmt1.concat_on(stmt2, " OR ");

        (stmt1, params1)
    }
}