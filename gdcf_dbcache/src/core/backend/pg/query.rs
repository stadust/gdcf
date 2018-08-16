use core::AsSql;
use core::backend::Database;
use core::backend::pg::Pg;
use core::query::create::{Column, Create};
use core::query::delete::Delete;
use core::query::Insert;
use core::query::insert::OnConflict;
use core::query::Select;
use core::query::select::Join;
use core::query::select::OrderBy;
use core::QueryPart;
use core::statement::{Preparation, Prepare, PreparedStatement};
use core::table::FieldValue;
use joinery::Joinable;
use core::query::select::Ordering;

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
    fn to_sql(&self) -> (PreparedStatement, Vec<&dyn AsSql<Pg>>) {
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
    fn to_sql(&self) -> (PreparedStatement, Vec<&dyn AsSql<Pg>>) {
        let mut p = Preparation::<Pg>::default()
            .with_static(self.name)
            .with_static(" ")
            .with_static(self.sql_type.to_raw_sql());

        for con in &self.constraints {
            p = p.with(con.to_sql())
                .with_static(" ")
        }

        p
    }
}

impl<'a> QueryPart<Pg> for Create<'a, Pg> {
    fn to_sql(&self) -> (PreparedStatement, Vec<&dyn AsSql<Pg>>) {
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

    fn to_sql(&self) -> (PreparedStatement, Vec<&dyn AsSql<Pg>>) {
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
    fn to_sql(&self) -> (PreparedStatement, Vec<&dyn AsSql<Pg>>) {
        Preparation::<Pg>::default()
            .with_static("JOIN ")
            .with_static(self.other.name)
            .with_static(" ON ")
            .with(self.join_condition.to_sql())
    }
}

impl<'a> QueryPart<Pg> for OrderBy<'a> {
    fn to_sql(&self) -> (PreparedStatement, Vec<&dyn AsSql<Pg>>) {
        let p = Preparation::<Pg>::default()
            .with_static(self.field.name);

        match self.ordering {
            Ordering::Asc => p.with_static("ASC"),
            Ordering::Desc => p.with_static("DESC")
        }
    }
}

impl<'a> QueryPart<Pg> for Delete<'a, Pg> {
    fn to_sql(&self) -> (PreparedStatement, Vec<&dyn AsSql<Pg>>) {
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
