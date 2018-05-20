use core::query::QueryPart;
use core::backend::pg::Pg;
use core::query::create::{PrimaryKeyConstraint, UniqueConstraint, NotNullConstraint};

simply_query_part!(Pg, PrimaryKeyConstraint<'a>, "PRIMARY KEY");
simply_query_part!(Pg, UniqueConstraint<'a>, "UNIQUE");
simply_query_part!(Pg, NotNullConstraint<'a>, "UNIQUE");