use core::AsSql;
use core::backend::Database;
use core::backend::pg::Pg;
use core::query::{Insert, QueryPart};
use core::query::create::{Column, Create};
use core::query::Select;
use core::query::select::Join;
use core::query::select::OrderBy;
use core::query::select::Ordering;
use core::statement::PreparedStatement;
use core::statement::StatementPart;
use core::table::Field;
use core::table::FieldValue;
use gdcf::ext::Join as __;

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

    fn to_sql<'b>(&'b self) -> (PreparedStatement, Vec<&'b AsSql<Pg>>) {
        let mut fields = Vec::new();
        let mut stmt = Vec::new();
        let mut values = Vec::new();

        for set_field in self.values() {
            fields.push(set_field.field.name());

            if let FieldValue::Value(value) = set_field.value {
                values.push(value);
                stmt.push(StatementPart::Placeholder);
            } else {
                stmt.push("DEFAULT".into());
            }

            stmt.push(",".into());
        }

        stmt.pop();  // Remove the last comma because it would cause a syntax error

        let mut stmt = PreparedStatement::new(stmt);

        stmt.prepend(format!("INSERT INTO {} ({}) VALUES (", self.table().name, fields.join(",")));
        stmt.append(")");

        (stmt, values)
    }
}

impl<'a> QueryPart<Pg> for Column<'a, Pg> {
    fn to_sql_unprepared(&self) -> String {
        let mut sql_constraints = Vec::new();

        for constraint in &self.constraints {
            sql_constraints.push(constraint.to_sql_unprepared())
        }

        format!("{} {} {}", self.name, self.sql_type.to_sql_unprepared(), sql_constraints.join(" "))
    }

    fn to_sql<'b>(&'b self) -> (PreparedStatement, Vec<&'b AsSql<Pg>>) {
        let mut prep_stmt = PreparedStatement::new(vec![self.name.into(), self.sql_type.to_sql_unprepared().into()]);
        let mut values = Vec::new();

        for constraint in &self.constraints {
            let (prep_cons, mut cons_values) = constraint.to_sql();
            prep_stmt.concat_on(prep_cons, " ");
            values.append(&mut cons_values);
        }

        (prep_stmt, values)
    }
}

impl<'a> QueryPart<Pg> for Create<'a, Pg> {
    fn to_sql_unprepared(&self) -> String {
        let mut column_sql = Vec::new();

        for column in &self.columns {
            column_sql.push(column.to_sql_unprepared())
        }

        format!(
            "CREATE TABLE {} {} ({})",
            if self.ignore_if_exists { "IF NOT EXISTS" } else { "" },
            self.name,
            column_sql.join(",")
        )
    }

    fn to_sql<'b>(&'b self) -> (PreparedStatement, Vec<&'b AsSql<Pg>>) {
        let mut values = Vec::new();
        let mut prep_stmt = PreparedStatement::new(
            vec![
                "CREATE TABLE".into(),
                if self.ignore_if_exists { "IF NOT EXISTS" } else { "" }.into(),
                self.name.into(),
                "(".into()
            ]
        );

        for column in &self.columns {
            let (mut prep_col, mut col_values) = column.to_sql();

            values.append(&mut col_values);
            prep_stmt.concat(prep_col);
            prep_stmt.append(",");
        }

        prep_stmt.pop(); // Remove the last comma because it would cause a syntax error
        prep_stmt.append(")");

        (prep_stmt, values)
    }
}

impl<'a> QueryPart<Pg> for Select<'a, Pg> {
    //TODO: implement
    fn to_sql_unprepared(&self) -> String {
        let qualify = !self.joins.is_empty();
        let where_clause = if let Some(ref condition) = self.filter {
            format!("WHERE {}", condition.to_sql_unprepared())
        } else {
            String::new()
        };

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

    fn to_sql<'b>(&'b self) -> (PreparedStatement, Vec<&'b AsSql<Pg>>) {
        unimplemented!()
    }
}

impl<'a> QueryPart<Pg> for Join<'a, Pg> {
    fn to_sql_unprepared(&self) -> String {
        format!("JOIN {} ON {}", self.other.name, self.join_condition.to_sql_unprepared())
    }

    fn to_sql<'b>(&'b self) -> (PreparedStatement, Vec<&'b AsSql<Pg>>) {
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

    fn to_sql<'b>(&'b self) -> (PreparedStatement, Vec<&'b AsSql<Pg>>) {
        unimplemented!()
    }
}

fn join_unprepared<DB: Database, QP: QueryPart<DB>>(parts: &[QP], sep: &str) -> String {
    let mut sql = Vec::new();

    for part in parts {
        sql.push(part.to_sql_unprepared())
    }

    sql.join(sep)
}