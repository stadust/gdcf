use core::AsSql;
use core::backend::Database;
use core::backend::pg::Pg;
use core::query::{Insert, QueryPart};
use core::query::create::{Column, Create};
use core::query::Select;
use core::query::select::Join;
use core::query::select::OrderBy;
use core::query::select::Ordering;
use core::statement::{Preparation, Prepare, PreparedStatement};
use core::table::FieldValue;
use gdcf::ext::Join as __;
// one underscore is unstable, but 2 are ok, kden

// TODO: ON CONFLICT UPDATE
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

        format!("INSERT INTO {} ({}) VALUES ({})", self.table().name, fields.join(","), values.join(","))
    }

    fn to_sql<'b>(&'b self) -> (PreparedStatement, Vec<&'b dyn AsSql<Pg>>) {
        let mut p = Preparation::<Pg>::default()
            .with_static("INSERT INTO ")
            .with_static(self.table().name);

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

        p.with(pv)
            .with_static(")")
    }
}

impl<'a> QueryPart<Pg> for Column<'a, Pg> {
    fn to_sql_unprepared(&self) -> String {
        format!("{} {} {}", self.name, self.sql_type.to_sql_unprepared(), self.constraints.iter().map(|c| c.to_sql_unprepared()).join(" "))
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
                .join(",")
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

impl<'a> QueryPart<Pg> for Select<'a, Pg> {
    //TODO: implement
    fn to_sql_unprepared(&self) -> String {
        let qualify = !self.joins.is_empty();
        let where_clause = self.filter.as_ref()
            .map_or(String::new(), |c| format!(" WHERE {}", c.to_sql_unprepared()));

        let join_clause = self.joins.iter()
            .map(|j| j.to_sql_unprepared())
            .join(" ");

        let order_clause = if !self.order.is_empty() {
            format!("ORDER BY {}", self.order.iter()
                .map(|o| o.to_sql_unprepared())
                .join(","))
        } else {
            String::new()
        };

        let bounds = match self.subset {
            (None, None) => String::new(),
            (Some(limit), None) => format!("LIMIT {}", limit),
            (None, Some(offset)) => format!("OFFSET {}", offset),
            (Some(limit), Some(offset)) => format!("LIMIT {} OFFSET {}", limit, offset)
        };

        let field_list = if qualify {
            self.fields.iter()
                .map(|f| f.qualified_name())
                .join(",")
        } else {
            self.fields.iter()
                .map(|f| f.name())
                .join(",")
        };

        format!("SELECT {} FROM {} {} {} {} {}", field_list, self.table.name, join_clause, where_clause, bounds, order_clause)
    }

    fn to_sql<'b>(&'b self) -> (PreparedStatement, Vec<&'b dyn AsSql<Pg>>) {
        unimplemented!()
    }
}

impl<'a> QueryPart<Pg> for Join<'a, Pg> {
    fn to_sql_unprepared(&self) -> String {
        format!("JOIN {} ON {}", self.other.name, self.join_condition.to_sql_unprepared())
    }

    fn to_sql<'b>(&'b self) -> (PreparedStatement, Vec<&'b dyn AsSql<Pg>>) {
        unimplemented!()
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
