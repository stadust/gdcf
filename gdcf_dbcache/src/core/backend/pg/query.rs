use core::AsSql;
use core::backend::pg::Pg;
use core::query::{Insert, Query, QueryPart};
use core::table::FieldValue;
use core::statement::PreparedStatement;
use core::statement::StatementPart;

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
            }else{
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

impl<'a> Query<'a, Pg> for Insert<'a, Pg> {}