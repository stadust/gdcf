use core::query::QueryPart;
use core::backend::pg::Pg;
use core::query::create::{PrimaryKeyConstraint, UniqueConstraint, NotNullConstraint};


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