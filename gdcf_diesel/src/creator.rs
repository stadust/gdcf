use crate::{meta::DatabaseEntry, wrap::Wrapped};
use diesel::{
    associations::{HasTable, Identifiable},
    backend::Backend,
    deserialize::FromSqlRow,
    dsl::Eq,
    query_builder::AsChangeset,
    sql_types::{Int8, Nullable, Text},
    ExpressionMethods, Insertable, Queryable,
};
use gdcf_model::user::Creator;

impl HasTable for Wrapped<Creator> {
    type Table = creator::table;

    fn table() -> Self::Table {
        creator::table
    }
}

impl<'a> Identifiable for &'a Wrapped<Creator> {
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

meta_table!(creator_meta, user_id);

type CreatorRow = (u64, String, Option<u64>);
type CreatorSqlType = (Int8, Text, Nullable<Int8>);
type CreatorValues<'a> = (
    Eq<creator::user_id, i64>,
    Eq<creator::name, &'a str>,
    Eq<creator::account_id, Option<i64>>,
);

fn values(creator: &Creator) -> CreatorValues {
    use creator::columns::*;

    (
        user_id.eq(creator.user_id as i64),
        name.eq(&creator.name[..]),
        account_id.eq(creator.account_id.map(|i| i as i64)),
    )
}

impl<DB: Backend> Queryable<CreatorSqlType, DB> for Wrapped<Creator>
where
    CreatorRow: FromSqlRow<CreatorSqlType, DB>,
{
    type Row = CreatorRow;

    fn build(row: Self::Row) -> Self {
        Wrapped(Creator {
            user_id: row.0,
            name: row.1,
            account_id: row.2,
        })
    }
}

impl<'a> Insertable<creator::table> for &'a Creator {
    type Values = <CreatorValues<'a> as Insertable<creator::table>>::Values;

    fn values(self) -> Self::Values {
        values(self).values()
    }
}

impl<'a> AsChangeset for Wrapped<&'a Creator> {
    type Changeset = <CreatorValues<'a> as AsChangeset>::Changeset;
    type Target = creator::table;

    fn as_changeset(self) -> Self::Changeset {
        values(&self.0).as_changeset()
    }
}
