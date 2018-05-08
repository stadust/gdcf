use core::AsSql;
use core::backend::pg::Pg;
use core::query::{Insert, Query, QueryPart};
use core::query::InsertValue;
use core::statement::PreparedStatement;
use core::statement::StatementPart;
use gdcf::ext::Join;

impl<'a> QueryPart<'a, Pg> for Insert<'a, Pg> {
    fn to_sql_unprepared(&self) -> String {
        let values = self.values()
            .iter()
            .map(|value| match value {
                InsertValue::Default => "DEFAULT".into(),
                InsertValue::Value(value) => value.as_sql_string()
            })
            .join(", ");

        format!("INSERT INTO {} VALUES ({})", self.table().name, values)
    }

    fn to_sql(&self) -> (PreparedStatement, Vec<&'a AsSql<Pg>>) {
        let parts = self.values()
            .iter()
            .map(|value| match value {
                InsertValue::Default => "DEFAULT".into(),
                InsertValue::Value(_) => StatementPart::Placeholder
            })
            .collect();

        let values = self.values()
            .iter()
            .filter_map(|value| match value {
                InsertValue::Default => None,
                InsertValue::Value(value) => Some(*value)
            })
            .collect();

        let mut stmt = PreparedStatement::new(parts);

        stmt.prepend(format!("INSERT INTO {} VALUES (", self.table().name));
        stmt.append(")");

        (stmt, values)
    }
}

impl<'a> Query<'a, Pg> for Insert<'a, Pg> {}