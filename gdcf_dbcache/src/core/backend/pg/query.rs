use core::AsSql;
use core::backend::pg::Pg;
use core::query::{Insert, QueryPart};
use core::query::create::{Column, Create};
use core::statement::PreparedStatement;
use core::statement::StatementPart;
use core::table::FieldValue;

impl<'a> QueryPart<'a, Pg> for Insert<'a, Pg> {
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

    fn to_sql(&self) -> (PreparedStatement, Vec<&'a AsSql<Pg>>) {
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

impl<'a> QueryPart<'a, Pg> for Column<'a, Pg> {
    fn to_sql_unprepared(&self) -> String {
        let mut sql_constraints = Vec::new();

        for constraint in &self.constraints {
            sql_constraints.push(constraint.to_sql_unprepared())
        }

        format!("{} {} {}", self.name, self.sql_type.to_sql_unprepared(), sql_constraints.join(" "))
    }

    fn to_sql(&self) -> (PreparedStatement, Vec<&'a AsSql<Pg>>) {
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

impl<'a> QueryPart<'a, Pg> for Create<'a, Pg> {
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

    fn to_sql(&self) -> (PreparedStatement, Vec<&'a AsSql<Pg>>) {
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
            prep_stmt.concat_on(prep_col, ",");
        }

        prep_stmt.append(")");

        (prep_stmt, values)
    }
}