use crate::core::{
    backend::{sqlite::Sqlite, util::join_statements},
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

constraint_query_part!(Sqlite, PrimaryKeyConstraint<'a>, "PRIMARY KEY");
constraint_query_part!(Sqlite, UniqueConstraint<'a>, "UNIQUE");
constraint_query_part!(Sqlite, NotNullConstraint<'a>, "NOT NULL");

impl<'a> QueryPart<Sqlite> for DefaultConstraint<'a, Sqlite> {
    fn to_sql(&self) -> Preparation<Sqlite> {
        match self.name {
            None =>
                Preparation::<Sqlite>::default()
                    .with_static("DEFAULT")
                    .with_static(self.default.to_raw_sql()),
            Some(_) => unimplemented!(),
        }
    }
}

simple_query_part!(Sqlite, Text, "TEXT");
simple_query_part!(Sqlite, SmallInteger, "INTEGER");
simple_query_part!(Sqlite, Integer, "INTEGER");
simple_query_part!(Sqlite, BigInteger, "INTEGER");
simple_query_part!(Sqlite, Boolean, "INTEGER");
simple_query_part!(Sqlite, Float, "REAL");
simple_query_part!(Sqlite, Double, "REAL");
simple_query_part!(Sqlite, Unsigned<SmallInteger>, "INTEGER");
simple_query_part!(Sqlite, Unsigned<Integer>, "INTEGER");
simple_query_part!(Sqlite, Unsigned<BigInteger>, "INTEGER");
simple_query_part!(Sqlite, UtcTimestamp, "TEXT");
simple_query_part!(Sqlite, Bytes, "BLOB");

impl QueryPart<Sqlite> for EqField {
    fn to_sql(&self) -> (PreparedStatement, Vec<&dyn AsSql<Sqlite>>) {
        Preparation::<Sqlite>::default()
            .with_static("(")
            .with_static(self.field_1.qualified_name())
            .with_static("=")
            .with_static(self.field_2.qualified_name())
            .with_static(")")
    }
}

impl QueryPart<Sqlite> for EqValue<Sqlite> {
    fn to_sql(&self) -> Preparation<Sqlite> {
        Preparation::<Sqlite>::default()
            .with_static("(")
            .with_static(self.field.qualified_name())
            .with_static("=")
            .with(self.value.to_sql())
            .with_static(")")
    }
}

impl QueryPart<Sqlite> for And<Sqlite> {
    fn to_sql(&self) -> (PreparedStatement, Vec<&dyn AsSql<Sqlite>>) {
        Preparation::<Sqlite>::default()
            .with_static("(")
            .with(self.cond_1.to_sql())
            .with_static("AND")
            .with(self.cond_2.to_sql())
            .with_static(")")
    }
}

impl QueryPart<Sqlite> for Or<Sqlite> {
    fn to_sql(&self) -> (PreparedStatement, Vec<&dyn AsSql<Sqlite>>) {
        Preparation::<Sqlite>::default()
            .with_static("(")
            .with(self.cond_1.to_sql())
            .with_static("OR")
            .with(self.cond_2.to_sql())
            .with_static(")")
    }
}

impl<'a> QueryPart<Sqlite> for Insert<'a, Sqlite> {
    fn to_sql(&self) -> (PreparedStatement, Vec<&dyn AsSql<Sqlite>>) {
        let p = match self.conflict {
            OnConflict::Ignore => Preparation::<Sqlite>::default().with_static("INSERT OR IGNORE"),
            OnConflict::Update(_) => Preparation::<Sqlite>::default().with_static("INSERT OR REPLACE"), /* TODO: maybe use actual */
            // UPSERT here
            OnConflict::Fail => Preparation::<Sqlite>::default().with_static("INSERT OR FAIL"),
        };

        let mut p = p.with_static("INTO").with_static(self.table().name).with_static("(");

        let mut pv = Preparation::<Sqlite>::default().with_static("VALUES (");

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

        p.with_static(")").with(pv).with_static(")")
    }
}

impl<'a> QueryPart<Sqlite> for Column<'a, Sqlite> {
    fn to_sql(&self) -> (PreparedStatement, Vec<&dyn AsSql<Sqlite>>) {
        let mut p = Preparation::<Sqlite>::default()
            .with_static(self.name)
            .with_static(self.sql_type.to_raw_sql());

        for con in &self.constraints {
            p = p.with(con.to_sql())
        }

        p
    }
}

impl<'a> QueryPart<Sqlite> for Create<'a, Sqlite> {
    fn to_sql(&self) -> (PreparedStatement, Vec<&dyn AsSql<Sqlite>>) {
        let mut p = Preparation::<Sqlite>::default().with_static("CREATE TABLE");

        if self.ignore_if_exists {
            p = p.with_static("IF NOT EXISTS");
        }

        p.with_static(self.name)
            .with_static("(")
            .with(join_statements(&self.columns, Some(",")))
            .with_static(")")
    }
}

impl Select<Sqlite> {
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

impl QueryPart<Sqlite> for Select<Sqlite> {
    fn to_sql(&self) -> (PreparedStatement, Vec<&dyn AsSql<Sqlite>>) {
        let mut p = Preparation::<Sqlite>::default()
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

impl QueryPart<Sqlite> for Join<Sqlite> {
    fn to_sql(&self) -> (PreparedStatement, Vec<&dyn AsSql<Sqlite>>) {
        Preparation::<Sqlite>::default()
            .with_static("JOIN")
            .with_static(self.other.name)
            .with_static("ON")
            .with(self.join_condition.to_sql())
    }
}

impl QueryPart<Sqlite> for OrderBy {
    fn to_sql(&self) -> (PreparedStatement, Vec<&dyn AsSql<Sqlite>>) {
        let p = Preparation::<Sqlite>::default().with_static(self.field.name);

        match self.ordering {
            Ordering::Asc => p.with_static("ASC"),
            Ordering::Desc => p.with_static("DESC"),
        }
    }
}

impl<'a> QueryPart<Sqlite> for Delete<'a, Sqlite> {
    fn to_sql(&self) -> (PreparedStatement, Vec<&dyn AsSql<Sqlite>>) {
        let mut p = Preparation::<Sqlite>::default()
            .with_static("DELETE FROM")
            .with_static(self.table.name);

        if let Some(ref filter) = self.filter {
            p = p.with_static("WHERE").with(filter.to_sql());
        }

        p
    }
}
