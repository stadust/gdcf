use diesel::{
    associations::{HasTable, Identifiable},
    backend::Backend,
    deserialize::FromSqlRow,
    dsl::Eq,
    sql_types::{Int8, Nullable, Text},
    ExpressionMethods, Insertable, Queryable,
};
use gdcf_model::user::Creator;

pub struct CreatorDB(Creator);

impl HasTable for CreatorDB {
    type Table = creator::table;

    fn table() -> Self::Table {
        creator::table
    }
}

impl<'a> Identifiable for &'a CreatorDB {
    type Id = &'a u64;

    fn id(self) -> Self::Id {
        &self.0.user_id
    }
}

table! {
    creator (user_id) {
        user_id -> Int8,
        name -> Text,
        account_id -> Nullable<Int8>,
    }
}

type CreatorRow = (u64, String, Option<u64>);
type CreatorSqlType = (Int8, Text, Nullable<Int8>);
type CreatorValues<'a> = (
    Eq<creator::user_id, i64>,
    Eq<creator::name, &'a str>,
    Eq<creator::account_id, Option<i64>>,
);

impl<DB: Backend> Queryable<CreatorSqlType, DB> for CreatorDB
where
    CreatorRow: FromSqlRow<CreatorSqlType, DB>,
{
    type Row = CreatorRow;

    fn build(row: Self::Row) -> Self {
        CreatorDB(Creator {
            user_id: row.0,
            name: row.1,
            account_id: row.2,
        })
    }
}

impl<'a> Insertable<creator::table> for &'a Creator {
    type Values = <CreatorValues<'a> as Insertable<creator::table>>::Values;

    fn values(self) -> Self::Values {
        use creator::columns::*;

        (
            user_id.eq(self.user_id as i64),
            name.eq(&self.name[..]),
            account_id.eq(self.account_id.map(|i| i as i64)),
        )
            .values()
    }
}
