use core::backend::pg::Pg;
use core::query::create::{NotNullConstraint, PrimaryKeyConstraint, UniqueConstraint};
use core::query::create::DefaultConstraint;
use core::QueryPart;
use core::statement::Preparation;
use core::statement::Prepare;


macro_rules! constraint_query_part {
    ($back: ty, $t: ty, $val: expr) => {
        impl<'a> QueryPart<$back> for $t {
            fn to_sql(&self) -> Preparation<$back> {
                match self.0 {
                    None => Preparation::<$back>::default().with_static($val),
                    Some(name) => Preparation::<$back>::default()
                        .with_static("CONSTRAINT")
                        .with_static(name)
                        .with_static($val)
                }
            }
        }
    };
}

constraint_query_part!(Pg, PrimaryKeyConstraint<'a>, "PRIMARY KEY");
constraint_query_part!(Pg, UniqueConstraint<'a>, "UNIQUE");
constraint_query_part!(Pg, NotNullConstraint<'a>, "NOT NULL");

impl<'a> QueryPart<Pg> for DefaultConstraint<'a, Pg> {
    fn to_sql(&self) -> Preparation<Pg> {
        match self.name {
            None => Preparation::<Pg>::default()
                .with_static("DEFAULT")
                .with_static(self.default.to_raw_sql()),
            Some(_) => unimplemented!()
        }
    }
}

