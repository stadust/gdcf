use chrono::{NaiveDateTime, Utc};
use diesel::{
    backend::Backend,
    deserialize::FromSqlRow,
    sql_types::{BigInt, Timestamp},
    Queryable,
};
use gdcf::cache::CacheEntryMeta;

#[derive(Debug, Clone, Copy)]
pub struct Entry {
    pub(crate) cached_at: NaiveDateTime,
    pub(crate) expired: bool,
    pub(crate) key: u64,
}

impl PartialEq for Entry {
    fn eq(&self, other: &Self) -> bool {
        self.key == other.key
    }
}

impl Eq for Entry {}

impl Entry {
    pub(crate) fn new(key: u64) -> Self {
        Self {
            cached_at: Utc::now().naive_utc(),
            expired: false,
            key,
        }
    }
}

impl CacheEntryMeta for Entry {
    fn is_expired(&self) -> bool {
        self.expired
    }
}

#[derive(Debug, Copy, Clone)]
pub(crate) struct DatabaseEntry {
    pub(crate) key: u64,
    pub(crate) cached_at: NaiveDateTime,
}

impl<DB: Backend> Queryable<(BigInt, Timestamp), DB> for DatabaseEntry
where
    (i64, NaiveDateTime): FromSqlRow<(BigInt, Timestamp), DB>,
{
    type Row = (i64, NaiveDateTime);

    fn build(row: Self::Row) -> Self {
        DatabaseEntry {
            key: row.0 as u64,
            cached_at: row.1,
        }
    }
}

macro_rules! meta_table {
    ($name: ident, $primary: ident) => {
        table! {
            $name($primary) {
                $primary -> Int8,
                cached_at -> Timestamp,
            }
        }

        impl diesel::Insertable<$name::table> for crate::meta::Entry {
            type Values = <(
                std::option::Option<diesel::dsl::Eq<$name::$primary, i64>>,
                std::option::Option<diesel::dsl::Eq<$name::cached_at, chrono::NaiveDateTime>>,
            ) as diesel::Insertable<$name::table>>::Values;

            fn values(self) -> Self::Values {
                use diesel::ExpressionMethods;

                (
                    std::option::Option::Some($name::$primary.eq(self.key as i64)),
                    std::option::Option::Some($name::cached_at.eq(self.cached_at)),
                )
                    .values()
            }
        }
    };
}
