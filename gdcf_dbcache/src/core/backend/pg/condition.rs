use core::{AsSql, query::{condition::{And, EqField, EqValue, Or}, QueryPart}, statement::{Preparation, Prepare, PreparedStatement}};
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
    }
}

impl QueryPart<Pg> for And<Pg> {
    fn to_sql_unprepared(&self) -> String {
        format!("({} AND {})", self.cond_1.to_sql_unprepared(), self.cond_2.to_sql_unprepared())
    }

    fn to_sql<'b>(&'b self) -> (PreparedStatement, Vec<&'b dyn AsSql<Pg>>) {
        Preparation::<Pg>::default()
            .with_static("(")
            .with(self.cond_1.to_sql())
            .with_static(" AND ")
            .with(self.cond_2.to_sql())
            .with_static(")")
    }
}


impl QueryPart<Pg> for Or<Pg> {
    fn to_sql_unprepared(&self) -> String {
        format!("({} OR {})", self.cond_1.to_sql_unprepared(), self.cond_2.to_sql_unprepared())
    }

    fn to_sql<'b>(&'b self) -> (PreparedStatement, Vec<&'b dyn AsSql<Pg>>) {
        Preparation::<Pg>::default()
            .with_static("(")
            .with(self.cond_1.to_sql())
            .with_static(" OR ")
            .with(self.cond_2.to_sql())
            .with_static(")")
    }
}