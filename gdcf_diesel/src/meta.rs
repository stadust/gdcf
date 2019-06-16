use chrono::{NaiveDateTime, Utc};
use diesel::{
    backend::Backend,
    deserialize::FromSqlRow,
    sql_types::{BigInt, Bool, Timestamp},
    Queryable,
};
use gdcf::cache::CacheEntryMeta;
use std::fmt::{Display, Formatter};

#[derive(Debug, Clone, Copy)]
pub struct Entry {
    pub(crate) cached_at: NaiveDateTime,
    pub(crate) expired: bool,
    pub(crate) key: u64,
    pub(crate) absent: bool,
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
            absent: false,
            key,
        }
    }

    pub(crate) fn absent(key: u64) -> Self {
        Self {
            cached_at: Utc::now().naive_utc(),
            expired: false,
            absent: true,
            key,
        }
    }

    pub fn cached_at(&self) -> NaiveDateTime {
        self.cached_at
    }
}

impl Display for Entry {
    fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
        write!(
            f,
            "Entry {}, cached at {} (expired: {}), absent: {}",
            self.key, self.cached_at, self.expired, self.absent
        )
    }
}

impl CacheEntryMeta for Entry {
    fn is_expired(&self) -> bool {
        self.expired
    }

    fn is_absent(&self) -> bool {
        self.absent
    }
}

#[derive(Debug, Copy, Clone)]
pub(crate) struct DatabaseEntry {
    pub(crate) key: u64,
    pub(crate) cached_at: NaiveDateTime,
    pub(crate) absent: bool,
}

impl<DB: Backend> Queryable<(BigInt, Timestamp, Bool), DB> for DatabaseEntry
where
    (i64, NaiveDateTime, bool): FromSqlRow<(BigInt, Timestamp, Bool), DB>,
{
    type Row = (i64, NaiveDateTime, bool);

    fn build(row: Self::Row) -> Self {
        DatabaseEntry {
            key: row.0 as u64,
            cached_at: row.1,
            absent: row.2,
        }
    }
}

macro_rules! meta_table {
    ($name: ident, $primary: ident) => {
        table! {
            $name($primary) {
                $primary -> Int8,
                cached_at -> Timestamp,
                absent -> Bool,
            }
        }

        impl diesel::Insertable<$name::table> for crate::meta::Entry {
            type Values = <(
                Option<diesel::dsl::Eq<$name::$primary, i64>>,
                Option<diesel::dsl::Eq<$name::cached_at, chrono::NaiveDateTime>>,
                Option<diesel::dsl::Eq<$name::absent, bool>>,
            ) as diesel::Insertable<$name::table>>::Values;

            fn values(self) -> Self::Values {
                use diesel::ExpressionMethods;

                (
                    Some($name::$primary.eq(self.key as i64)),
                    Some($name::cached_at.eq(self.cached_at)),
                    Some($name::absent.eq(self.absent)),
                )
                    .values()
            }
        }
    };
}
