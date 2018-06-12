use core::query::QueryPart;
use core::backend::pg::Pg;
use core::query::create::{PrimaryKeyConstraint, UniqueConstraint, NotNullConstraint};
use core::query::create::DefaultConstraint;
use core::statement::Preparation;
use core::statement::Prepare;
use core::query::create::Constraint;
use core::SqlExpr;


macro_rules! constraint_query_part {
    ($back: ty, $t: ty, $val: expr) => {
        impl<'a> QueryPart<$back> for $t {
            fn to_sql_unprepared(&self) -> String {
                match self.0 {
                    None => String::from($val),
                    Some(name) => format!("CONSTRAINT {} {}", name, $val)
                }
            }
        }
    };
}

constraint_query_part!(Pg, PrimaryKeyConstraint<'a>, "PRIMARY KEY");
constraint_query_part!(Pg, UniqueConstraint<'a>, "UNIQUE");
constraint_query_part!(Pg, NotNullConstraint<'a>, "NOT NULL");

impl<'a, D: SqlExpr<Pg>> QueryPart<Pg> for DefaultConstraint<'a, Pg, D> {
    fn to_sql_unprepared(&self) -> String {
        match self.name {
            None => format!("DEFAULT {}", self.default.to_sql_unprepared()),
            Some(_) => unimplemented!()
        }
    }

    fn to_sql<'b>(&'b self) -> Preparation<'b, Pg> {
        match self.name {
            None => Preparation::<Pg>::default()
                .with_static("DEFAULT ")
                .with(self.default.to_sql()),
            Some(_) => unimplemented!()
        }
    }
}

