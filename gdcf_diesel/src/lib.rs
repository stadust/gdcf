#[deny(unused_must_use)]
#[macro_use]
mod meta;
mod creator;
mod song;
mod wrap;

// diesel devs refuse to make their macros work with the new rust 2018 import mechanics, so this
// shit is necessary

#[macro_use]
extern crate diesel;
#[macro_use]
extern crate diesel_migrations;

use crate::{
    meta::{DatabaseEntry, Entry},
    song::newgrounds_song,
    wrap::Wrapped,
};
use chrono::{DateTime, Duration, NaiveDateTime, Utc};
use diesel::{query_source::Table, r2d2::ConnectionManager, Connection, Insertable, RunQueryDsl};
use failure::Fail;
use gdcf::{
    cache::{CacheEntry, CacheEntryMeta, Lookup, Store},
    error::CacheError,
    Secondary,
};
use gdcf_model::{level::PartialLevel, song::NewgroundsSong, user::Creator};
use r2d2::Pool;

pub struct Cache<T: Connection + 'static> {
    pool: Pool<ConnectionManager<T>>,
    expire_after: Duration,
}

impl<T: Connection + 'static> Cache<T> {
    fn entry(&self, db_entry: DatabaseEntry) -> Entry {
        let now = Utc::now();
        let then = DateTime::<Utc>::from_utc(db_entry.cached_at, Utc);
        let expired = now - then > self.expire_after;

        Entry {
            expired,
            key: db_entry.key as u64,
            cached_at: db_entry.cached_at,
        }
    }
}

impl<T: Connection + 'static> Clone for Cache<T> {
    fn clone(&self) -> Self {
        Cache {
            pool: self.pool.clone(),
            expire_after: self.expire_after,
        }
    }
}

// this means we cannot enable two features at once. Since diesel doesn't allow writing database
// agnostic code, the alternative to this is wrapping everything in macaros (like we used to do in
// gdcf_dbcache). That's a crappy alternative. We dont do that there
#[cfg(feature = "pg")]
type DB = diesel::pg::PgConnection;

#[cfg(feature = "sqlite")]
type DB = diesel::sqlite::SqliteConnection;

#[cfg(feature = "pg")]
mod postgres {
    use super::Cache;
    use diesel::{connection::Connection, pg::PgConnection, r2d2::ConnectionManager};
    use r2d2::Pool;

    embed_migrations!("migrations/postgres");

    impl Cache<PgConnection> {
        pub fn postgres(database_url: String) -> Result<Self, r2d2::Error> {
            Pool::new(ConnectionManager::new(database_url)).map(Self)
        }

        pub fn initialize(&self) -> Result<(), diesel_migrations::RunMigrationsError> {
            embedded_migrations::run(&self.0.get().unwrap())
        }
    }
}

#[cfg(feature = "sqlite")]
mod sqlite {
    use super::Cache;
    use chrono::Duration;
    use diesel::{r2d2::ConnectionManager, sqlite::SqliteConnection};
    use r2d2::Pool;
    use std::path::Path;

    embed_migrations!("migrations/sqlite");

    impl Cache<SqliteConnection> {
        pub fn in_memory() -> Result<Self, r2d2::Error> {
            Ok(Self {
                pool: Pool::new(ConnectionManager::new(":memory:"))?,
                expire_after: Duration::seconds(60),
            })
        }

        pub fn sqlite(path: String) -> Result<Self, r2d2::Error> {
            Ok(Self {
                pool: Pool::new(ConnectionManager::new(path))?,
                expire_after: Duration::seconds(60),
            })
        }

        pub fn initialize(&self) -> Result<(), diesel_migrations::RunMigrationsError> {
            embedded_migrations::run(&self.pool.get().unwrap())
        }
    }
}

#[derive(Debug, Fail)]
pub enum Error {
    #[fail(display = "Not in database :(")]
    CacheMiss,

    #[fail(display = "Database error: {}", _0)]
    Database(#[cause] diesel::result::Error),

    #[fail(display = "Connection pool error: {}", _0)]
    R2D2(#[cause] r2d2::Error),
}

impl From<r2d2::Error> for Error {
    fn from(err: r2d2::Error) -> Self {
        Error::R2D2(err)
    }
}

impl From<diesel::result::Error> for Error {
    fn from(err: diesel::result::Error) -> Self {
        Error::Database(err)
    }
}

impl CacheError for Error {
    fn is_cache_miss(&self) -> bool {
        match self {
            Error::CacheMiss | Error::Database(diesel::result::Error::NotFound) => true,
            _ => false,
        }
    }
}

impl gdcf::cache::Cache for Cache<DB> {
    type CacheEntryMeta = Entry;
    type Err = Error;
}

#[cfg(feature = "pg")]
macro_rules! upsert {
    ($self: expr, $object: expr, $table: expr, $column: expr) => {{
        use diesel::{pg::upsert::*, query_builder::AsChangeset};

        diesel::insert_into($table)
            .values($object)
            .on_conflict($column)
            .do_update()
            .set(Wrapped($object))
            .execute(&$self.pool.get()?)?;
    }};
}

#[cfg(feature = "sqlite")]
macro_rules! upsert {
    ($self: expr, $object: expr, $table: expr, $_: expr) => {
        diesel::replace_into($table).values($object).execute(&$self.pool.get()?)?;
    };
}

macro_rules! store_simply {
    ($to_store_ty: ty, $table: ident, $meta: ident, $primary: ident) => {
        impl Store<$to_store_ty> for Cache<DB> {
            fn store(&mut self, object: &$to_store_ty, key: u64) -> Result<Entry, Self::Err> {
                let entry = Entry::new(key);

                entry.insert_into($meta::table).execute(&self.pool.get()?)?;

                upsert!(self, object, $table::table, $table::$primary);

                Ok(entry)
            }
        }
    };
}

macro_rules! lookup_simply {
    ($to_lookup_ty: ty, $object_table: ident,  $meta_table: ident, $primary_column: ident) => {
        impl Lookup<$to_lookup_ty> for Cache<DB> {
            fn lookup(&self, key: u64) -> Result<CacheEntry<$to_lookup_ty, Self>, Self::Err> {
                let connection = self.pool.get()?;
                let entry = $meta_table::table
                    .filter($meta_table::$primary_column.eq(key as i64))
                    .get_result(&connection)?;
                let entry = self.entry(entry);
                let wrapped: Wrapped<$to_lookup_ty> = $object_table::table
                    .filter($object_table::$primary_column.eq(key as i64))
                    .get_result(&connection)?;

                Ok(CacheEntry::new(wrapped.0, entry))
            }
        }
    };
}

use crate::{
    creator::{creator as cr, creator_meta as cr_m},
    song::{newgrounds_song as nr, song_meta as nr_m},
};
use diesel::{query_dsl::QueryDsl, ExpressionMethods};

store_simply!(Creator, cr, cr_m, user_id);
lookup_simply!(Creator, cr, cr_m, user_id);

store_simply!(NewgroundsSong, nr, nr_m, song_id);
lookup_simply!(NewgroundsSong, nr, nr_m, song_id);

impl Lookup<Vec<PartialLevel<u64, u64>>> for Cache<DB> {
    fn lookup(&self, key: u64) -> Result<CacheEntry<Vec<PartialLevel<u64, u64>>, Self>, Self::Err> {
        unimplemented!()
    }
}

impl Store<Vec<PartialLevel<u64, u64>>> for Cache<DB> {
    fn store(&mut self, obj: &Vec<PartialLevel<u64, u64>>, key: u64) -> Result<Self::CacheEntryMeta, Self::Err> {
        unimplemented!()
    }
}
