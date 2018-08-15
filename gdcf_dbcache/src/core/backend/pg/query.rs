use core::AsSql;
use core::backend::Database;
use core::backend::pg::Pg;
use core::query::{Insert, QueryPart};
use core::query::create::{Column, Create};
use core::query::delete::Delete;
use core::query::insert::OnConflict;
use core::query::Select;
use core::query::select::Join;
use core::query::select::OrderBy;
use core::query::select::Ordering;
use core::statement::{Preparation, Prepare, PreparedStatement};
use core::table::FieldValue;
use joinery::Joinable;

impl<'a> Insert<'a, Pg> {
    fn on_conflict(&self) -> String {
        match self.conflict {
            OnConflict::Ignore => "ON CONFLICT DO NOTHING".into(),
            OnConflict::Update(ref target) => {
                format!(
                    "ON CONFLICT ({}) DO UPDATE SET {}",
                    target
                        .iter()
                        .map(|f| f.name())
                        .join_with(","),
                    self.values()
                        .into_iter()
                        .map(|f| format!("{0}=EXCLUDED.{0}", f.field.name())).join_with(",")
                )
            }
            _ => String::new()
        }
    }
}

impl<'a> QueryPart<Pg> for Insert<'a, Pg> {
    fn to_sql_unprepared(&self) -> String {
        let mut fields = Vec::new();
        let mut values = Vec::new();

        for set_field in self.values() {
            fields.push(set_field.field.name());

            values.push(match set_field.value {
                FieldValue::Default => "DEFAULT".into(),
                FieldValue::Value(value) => value.as_sql_string()
            })
        }

        format!("INSERT INTO {} ({}) VALUES ({}) {}", self.table().name, fields.join_with(","), values.join_with(","), self.on_conflict())
    }

    fn to_sql<'b>(&'b self) -> (PreparedStatement, Vec<&'b dyn AsSql<Pg>>) {
        let mut p = Preparation::<Pg>::default()
            .with_static("INSERT INTO ")
            .with_static(self.table().name)
            .with_static("(");

        let mut pv = Preparation::<Pg>::default()
            .with_static(" VALUES (");

        for set_field in self.values() {
            p = p.with_static(set_field.field.name())
                .with_static(",");

            pv = match set_field.value {
                FieldValue::Default => pv.with_static("DEFAULT"),
                FieldValue::Value(v) => pv.with(v.to_sql())
            }.with_static(",");
        }

        p.0.pop();
        pv.0.pop();

        p.with_static(")")
            .with(pv)
            .with_static(")")
            .with_static(self.on_conflict())
    }
}

impl<'a> QueryPart<Pg> for Column<'a, Pg> {
    fn to_sql_unprepared(&self) -> String {
        format!("{} {} {}", self.name, self.sql_type.to_sql_unprepared(), self.constraints.iter().map(|c| c.to_sql_unprepared()).join_with(" "))
    }

    fn to_sql<'b>(&'b self) -> (PreparedStatement, Vec<&'b dyn AsSql<Pg>>) {
        let mut p = Preparation::<Pg>::default()
            .with_static(self.name)
            .with_static(" ")
            .with_static(self.sql_type.to_sql_unprepared());

        for con in &self.constraints {
            p = p.with(con.to_sql())
                .with_static(" ")
        }

        p
    }
}

impl<'a> QueryPart<Pg> for Create<'a, Pg> {
    fn to_sql_unprepared(&self) -> String {
        format!(
            "CREATE TABLE {} {} ({})",
            if self.ignore_if_exists { "IF NOT EXISTS" } else { "" },
            self.name,
            self.columns.iter()
                .map(|c| c.to_sql_unprepared())
                .join_with(",")
        )
    }

    fn to_sql<'b>(&'b self) -> (PreparedStatement, Vec<&'b dyn AsSql<Pg>>) {
        let mut p = Preparation::<Pg>::default()
            .with_static("CREATE TABLE ");

        if self.ignore_if_exists {
            p = p.with_static("IF NOT EXISTS ");
        }

        p.with_static(self.name)
            .with_static("(")
            .with(join_statements(&self.columns, ","))
            .with_static(")")
    }
}

impl<'a> Select<'a, Pg> {
    fn qualify(&self) -> bool {
        !self.joins.is_empty()
    }

    fn fields(&self) -> String {
        if self.qualify() {
            self.fields.iter()
                .map(|f| f.qualified_name())
                .join_with(",")
                .to_string()
        } else {
            self.fields.iter()
                .map(|f| f.name())
                .join_with(",")
                .to_string()
        }
    }

    fn bounds(&self) -> String {
        match self.subset {
            (None, None) => String::new(),
            (Some(limit), None) => format!("LIMIT {}", limit),
            (None, Some(offset)) => format!("OFFSET {}", offset),
            (Some(limit), Some(offset)) => format!("LIMIT {} OFFSET {}", limit, offset)
        }
    }
}

impl<'a> QueryPart<Pg> for Select<'a, Pg> {
    fn to_sql_unprepared(&self) -> String {
        let where_clause = self.filter.as_ref()
            .map_or(String::new(), |c| format!(" WHERE {}", c.to_sql_unprepared()));

        let join_clause = self.joins.iter()
            .map(|j| j.to_sql_unprepared())
            .join_with(" ");

        let order_clause = if !self.order.is_empty() {
            format!("ORDER BY {}", self.order.iter()
                .map(|o| o.to_sql_unprepared())
                .join_with(","))
        } else {
            String::new()
        };

        format!("SELECT {} FROM {} {} {} {} {}", self.fields(), self.table.name, join_clause, where_clause, self.bounds(), order_clause)
    }

    fn to_sql<'b>(&'b self) -> (PreparedStatement, Vec<&'b dyn AsSql<Pg>>) {
        let mut p = Preparation::<Pg>::default()
            .with_static("SELECT ")
            .with_static(self.fields())
            .with_static(" FROM ")
            .with_static(self.table.name)
            .with(join_statements(&self.joins, " "));

        if let Some(ref cond) = self.filter {
            p = p.with_static(" WHERE ")
                .with(cond.to_sql());
        }

        p.with_static(self.bounds())
            .with(join_statements(&self.order, ","))
    }
}

impl<'a> QueryPart<Pg> for Join<'a, Pg> {
    fn to_sql_unprepared(&self) -> String {
        format!("JOIN {} ON {}", self.other.name, self.join_condition.to_sql_unprepared())
    }

    fn to_sql<'b>(&'b self) -> (PreparedStatement, Vec<&'b dyn AsSql<Pg>>) {
        Preparation::<Pg>::default()
            .with_static("JOIN ")
            .with_static(self.other.name)
            .with_static(" ON ")
            .with(self.join_condition.to_sql())
    }
}

impl<'a> QueryPart<Pg> for OrderBy<'a> {
    fn to_sql_unprepared(&self) -> String {
        match self.ordering {
            Ordering::Asc => format!("{} ASC", self.field.name),
            Ordering::Desc => format!("{} DESC", self.field.name)
        }
    }

    fn to_sql<'b>(&'b self) -> (PreparedStatement, Vec<&'b dyn AsSql<Pg>>) {
        unimplemented!()
    }
}

impl<'a> QueryPart<Pg> for Delete<'a, Pg> {
    fn to_sql_unprepared(&self) -> String {
        match self.filter {
            Some(ref filter) => format!("DELETE FROM {} WHERE {}", self.table.name, filter.to_sql_unprepared()),
            None => format!("DELETE FROM {}", self.table.name)
        }
    }

    fn to_sql<'b>(&'b self) -> (PreparedStatement, Vec<&'b dyn AsSql<Pg>>) {
        let mut p = Preparation::<Pg>::default()
            .with_static("DELETE FROM ")
            .with_static(self.table.name);

        if let Some(ref filter) = self.filter {
            p = p.with_static(" WHERE ")
                .with(filter.to_sql());
        }

        p
    }
}

pub fn join_statements<'a, DB: 'a, QP: 'a, I>(stmts: I, seperator: &str) -> Preparation<'a, DB>
    where
        DB: Database,
        QP: QueryPart<DB>,
        I: IntoIterator<Item=&'a QP>
{
    let mut p = Preparation::<DB>::default();
    let mut sep = "";

    for t in stmts {
        p = p.with_static(sep)
            .with(t.to_sql());

        sep = seperator;
    }

    p
}
