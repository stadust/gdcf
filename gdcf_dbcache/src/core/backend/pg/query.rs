use crate::core::{
    backend::{pg::Pg, util::join_statements},
    query::{
        condition::{And, EqField, EqValue, Or},
        create::{Column, Create, DefaultConstraint, NotNullConstraint, PrimaryKeyConstraint, UniqueConstraint},
        delete::Delete,
        insert::OnConflict,
        select::{Join, OrderBy, Ordering},
        Insert, Select,
    },
    statement::{Preparation, Prepare, PreparedStatement},
    table::FieldValue,
    types::{BigInteger, Boolean, Bytes, Double, Float, Integer, SmallInteger, Text, Unsigned, UtcTimestamp},
    AsSql, QueryPart,
};
use joinery::Joinable;

macro_rules! constraint_query_part {
    ($back: ty, $t: ty, $val: expr) => {
        impl<'a> QueryPart<$back> for $t {
            fn to_sql(&self) -> Preparation<$back> {
                match self.0 {
                    None => Preparation::<$back>::default().with_static($val),
                    Some(name) =>
                        Preparation::<$back>::default()
                            .with_static("CONSTRAINT")
                            .with_static(name)
                            .with_static($val),
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
            None =>
                Preparation::<Pg>::default()
                    .with_static("DEFAULT")
                    .with_static(self.default.to_raw_sql()),
            Some(_) => unimplemented!(),
        }
    }
}

simple_query_part!(Pg, Text, "TEXT");
simple_query_part!(Pg, SmallInteger, "SMALLINT");
simple_query_part!(Pg, Integer, "INT");
simple_query_part!(Pg, BigInteger, "BIGINT");
simple_query_part!(Pg, Boolean, "BOOL");
simple_query_part!(Pg, Float, "FLOAT(4)");
simple_query_part!(Pg, Double, "DOUBLE PRECISION");
simple_query_part!(Pg, Unsigned<SmallInteger>, "SMALLINT");
simple_query_part!(Pg, Unsigned<Integer>, "INTEGER");
simple_query_part!(Pg, Unsigned<BigInteger>, "BIGINT");
simple_query_part!(Pg, UtcTimestamp, "TIMESTAMP WITHOUT TIME ZONE");
simple_query_part!(Pg, Bytes, "BYTEA");

impl QueryPart<Pg> for EqField {
    fn to_sql(&self) -> (PreparedStatement, Vec<&dyn AsSql<Pg>>) {
        Preparation::<Pg>::default()
            .with_static("(")
            .with_static(self.field_1.qualified_name())
            .with_static("=")
            .with_static(self.field_2.qualified_name())
            .with_static(")")
    }
}

impl QueryPart<Pg> for EqValue<Pg> {
    fn to_sql(&self) -> Preparation<Pg> {
        Preparation::<Pg>::default()
            .with_static("(")
            .with_static(self.field.qualified_name())
            .with_static("=")
            .with(self.value.to_sql())
            .with_static(")")
    }
}

impl QueryPart<Pg> for And<Pg> {
    fn to_sql(&self) -> (PreparedStatement, Vec<&dyn AsSql<Pg>>) {
        Preparation::<Pg>::default()
            .with_static("(")
            .with(self.cond_1.to_sql())
            .with_static("AND")
            .with(self.cond_2.to_sql())
            .with_static(")")
    }
}

impl QueryPart<Pg> for Or<Pg> {
    fn to_sql(&self) -> (PreparedStatement, Vec<&dyn AsSql<Pg>>) {
        Preparation::<Pg>::default()
            .with_static("(")
            .with(self.cond_1.to_sql())
            .with_static("OR")
            .with(self.cond_2.to_sql())
            .with_static(")")
    }
}

impl<'a> Insert<'a, Pg> {
    fn on_conflict(&self) -> String {
        match self.conflict {
            OnConflict::Ignore => "ON CONFLICT DO NOTHING".into(),
            OnConflict::Update(ref target) =>
                format!(
                    "ON CONFLICT ({}) DO UPDATE SET {}",
                    target.iter().map(|f| f.name()).join_with(","),
                    self.values()
                        .iter()
                        .map(|f| format!("{0}=EXCLUDED.{0}", f.field.name()))
                        .join_with(",")
                ),
            _ => String::new(),
        }
    }
}

impl<'a> QueryPart<Pg> for Insert<'a, Pg> {
    fn to_sql(&self) -> (PreparedStatement, Vec<&dyn AsSql<Pg>>) {
        let mut p = Preparation::<Pg>::default()
            .with_static("INSERT INTO")
            .with_static(self.table().name)
            .with_static("(");

        let mut pv = Preparation::<Pg>::default().with_static("VALUES (");

        for set_field in self.values() {
            p = p.with_static(set_field.field.name()).with_static(",");

            pv = match set_field.value {
                FieldValue::Default => pv.with_static("DEFAULT"),
                FieldValue::Value(v) => pv.with(v.to_sql()),
            }
            .with_static(",");
        }

        p.0.pop();
        pv.0.pop();

        p.with_static(")").with(pv).with_static(")").with_static(self.on_conflict())
    }
}

impl<'a> QueryPart<Pg> for Column<'a, Pg> {
    fn to_sql(&self) -> (PreparedStatement, Vec<&dyn AsSql<Pg>>) {
        let mut p = Preparation::<Pg>::default()
            .with_static(self.name)
            .with_static(self.sql_type.to_raw_sql());

        for con in &self.constraints {
            p = p.with(con.to_sql())
        }

        p
    }
}

impl<'a> QueryPart<Pg> for Create<'a, Pg> {
    fn to_sql(&self) -> (PreparedStatement, Vec<&dyn AsSql<Pg>>) {
        let mut p = Preparation::<Pg>::default().with_static("CREATE TABLE");

        if self.ignore_if_exists {
            p = p.with_static("IF NOT EXISTS");
        }

        p.with_static(self.name)
            .with_static("(")
            .with(join_statements(&self.columns, Some(",")))
            .with_static(")")
    }
}

impl Select<Pg> {
    fn qualify(&self) -> bool {
        !self.joins.is_empty()
    }

    fn fields(&self) -> String {
        if self.qualify() {
            self.fields.iter().map(|f| f.qualified_name()).join_with(",").to_string()
        } else {
            self.fields.iter().map(|f| f.name()).join_with(",").to_string()
        }
    }

    fn bounds(&self) -> String {
        match self.subset {
            (None, None) => String::new(),
            (Some(limit), None) => format!("LIMIT {}", limit),
            (None, Some(offset)) => format!("OFFSET {}", offset),
            (Some(limit), Some(offset)) => format!("LIMIT {} OFFSET {}", limit, offset),
        }
    }
}

impl QueryPart<Pg> for Select<Pg> {
    fn to_sql(&self) -> (PreparedStatement, Vec<&dyn AsSql<Pg>>) {
        let mut p = Preparation::<Pg>::default()
            .with_static("SELECT")
            .with_static(self.fields())
            .with_static("FROM")
            .with_static(self.table.name)
            .with(join_statements(&self.joins, None));

        if let Some(ref cond) = self.filter {
            p = p.with_static("WHERE").with(cond.to_sql());
        }

        p.with_static(self.bounds()).with(join_statements(&self.order, Some(",")))
    }
}

impl QueryPart<Pg> for Join<Pg> {
    fn to_sql(&self) -> (PreparedStatement, Vec<&dyn AsSql<Pg>>) {
        Preparation::<Pg>::default()
            .with_static("JOIN")
            .with_static(self.other.name)
            .with_static("ON")
            .with(self.join_condition.to_sql())
    }
}

impl QueryPart<Pg> for OrderBy {
    fn to_sql(&self) -> (PreparedStatement, Vec<&dyn AsSql<Pg>>) {
        let p = Preparation::<Pg>::default().with_static(self.field.name);

        match self.ordering {
            Ordering::Asc => p.with_static("ASC"),
            Ordering::Desc => p.with_static("DESC"),
        }
    }
}

impl<'a> QueryPart<Pg> for Delete<'a, Pg> {
    fn to_sql(&self) -> (PreparedStatement, Vec<&dyn AsSql<Pg>>) {
        let mut p = Preparation::<Pg>::default().with_static("DELETE FROM").with_static(self.table.name);

        if let Some(ref filter) = self.filter {
            p = p.with_static("WHERE").with(filter.to_sql());
        }

        p
    }
}
