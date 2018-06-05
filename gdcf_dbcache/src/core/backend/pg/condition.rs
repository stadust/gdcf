use core::{AsSql, query::{condition::{And, EqField, EqValue, Or}, QueryPart}, statement::{Preparation, Prepare, PreparedStatement, StatementPart}};
use super::Pg;

impl<'a> QueryPart<Pg> for EqField<'a> {
    fn to_sql_unprepared(&self) -> String {
        format!("({} = {})", self.field_1.qualified_name(), self.field_2.qualified_name())
    }
}

impl<'a> QueryPart<Pg> for EqValue<'a, Pg> {
    fn to_sql_unprepared(&self) -> String {
        format!("({} = {})", self.field.qualified_name(), self.value.as_sql_string())
    }

    fn to_sql<'b>(&'b self) -> Preparation<Pg> {
        Preparation::<Pg>::default()
            .with_static(format!("({} = ", self.field.qualified_name()))
            .with(self.value.to_sql())
            .with_static(")")
        /*let stmt = PreparedStatement::new(vec![
            format!("({} = ", self.field.qualified_name()).into(),
            StatementPart::Placeholder,
            ")".into()
        ]);

        (stmt, vec![&*self.value])*/
    }
}

impl QueryPart<Pg> for And<Pg> {
    fn to_sql_unprepared(&self) -> String {
        format!("({} AND {})", self.cond_1.to_sql_unprepared(), self.cond_2.to_sql_unprepared())
    }

    fn to_sql<'b>(&'b self) -> (PreparedStatement, Vec<&'b AsSql<Pg>>) {
        Preparation::<Pg>::default()
            .with_static("(")
            .with(self.cond_1.to_sql())
            .with_static(" AND ")
            .with(self.cond_2.to_sql())
            .with_static(")")
        /*let (mut stmt1, mut params1) = self.cond_1.to_sql();
        let (mut stmt2, mut params2) = self.cond_1.to_sql();

        params1.append(&mut params2);

        stmt1.prepend("(");
        stmt2.append(")");

        stmt1.concat_on(stmt2, " AND ");

        (stmt1, params1)*/
    }
}


impl QueryPart<Pg> for Or<Pg> {
    fn to_sql_unprepared(&self) -> String {
        format!("({} OR {})", self.cond_1.to_sql_unprepared(), self.cond_2.to_sql_unprepared())
    }

    fn to_sql<'b>(&'b self) -> (PreparedStatement, Vec<&'b AsSql<Pg>>) {
        Preparation::<Pg>::default()
            .with_static("(")
            .with(self.cond_1.to_sql())
            .with_static(" OR ")
            .with(self.cond_2.to_sql())
            .with_static(")")
        /*let (mut stmt1, mut params1) = self.cond_1.to_sql();
        let (mut stmt2, mut params2) = self.cond_1.to_sql();

        params1.append(&mut params2);

        stmt1.prepend("(");
        stmt2.append(")");

        stmt1.concat_on(stmt2, " OR ");

        (stmt1, params1)*/
    }
}