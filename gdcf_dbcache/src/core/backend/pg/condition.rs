use core::{AsSql, QueryPart, query::condition::{And, EqField, EqValue, Or}, statement::{Preparation, Prepare, PreparedStatement}};
use super::Pg;

impl<'a> QueryPart<Pg> for EqField<'a> {
    fn to_sql(&self) -> (PreparedStatement, Vec<&dyn AsSql<Pg>>) {
        Preparation::<Pg>::default()
            .with_static("(")
            .with_static(self.field_1.qualified_name())
            .with_static("=")
            .with_static(self.field_2.qualified_name())
            .with_static(")")
    }
}

impl<'a> QueryPart<Pg> for EqValue<'a, Pg> {
    fn to_sql(&self) -> Preparation<Pg> {
        Preparation::<Pg>::default()
            .with_static(format!("({} = ", self.field.qualified_name()))
            .with(self.value.to_sql())
            .with_static(")")
    }
}

impl QueryPart<Pg> for And<Pg> {
    fn to_sql(&self) -> (PreparedStatement, Vec<&dyn AsSql<Pg>>) {
        Preparation::<Pg>::default()
            .with_static("(")
            .with(self.cond_1.to_sql())
            .with_static(" AND ")
            .with(self.cond_2.to_sql())
            .with_static(")")
    }
}


impl QueryPart<Pg> for Or<Pg> {
    fn to_sql(&self) -> (PreparedStatement, Vec<&dyn AsSql<Pg>>) {
        Preparation::<Pg>::default()
            .with_static("(")
            .with(self.cond_1.to_sql())
            .with_static(" OR ")
            .with(self.cond_2.to_sql())
            .with_static(")")
    }
}