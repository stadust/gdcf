#[deny(unused_must_use)]
#[macro_use]
mod meta;
#[macro_use]
mod macros;
mod creator;
mod level;
mod partial_level;
mod song;
mod wrap;

// diesel devs refuse to make their macros work with the new rust 2018 import mechanics, so this
// shit is necessary

#[macro_use]
extern crate diesel;
#[macro_use]
extern crate diesel_migrations;

use crate::{
    creator::{creator as cr, creator_meta as cr_m},
    meta::{DatabaseEntry, Entry},
    song::{newgrounds_song as nr, song_meta as nr_m},
    wrap::Wrapped,
};
use chrono::{DateTime, Duration, NaiveDateTime, Utc};
use diesel::{query_dsl::QueryDsl, query_source::Table, r2d2::ConnectionManager, Connection, ExpressionMethods, Insertable, RunQueryDsl};
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

// TODO: in the future we can probably make these macro-generated as well, but for now we only have
// one of them, so its fine

impl Lookup<Vec<PartialLevel<u64, u64>>> for Cache<DB> {
    fn lookup(&self, key: u64) -> Result<CacheEntry<Vec<PartialLevel<u64, u64>>, Self>, Self::Err> {
        use crate::partial_level::*;
        use diesel::{JoinOnDsl, Queryable};

        let connection = self.pool.get()?;

        let entry: DatabaseEntry = level_list_meta::table
            .filter(level_list_meta::request_hash.eq(key as i64))
            .get_result(&connection)?;

        let levels = partial_level::table
            .inner_join(request_results::table.on(partial_level::level_id.eq(request_results::level_id)))
            .filter(request_results::request_hash.eq(key as i64))
            .select(partial_level::all_columns)
            .load(&connection)?
            .into_iter()
            .map(|row: Wrapped<_>| row.0)
            .collect();

        Ok(CacheEntry::new(levels, self.entry(entry)))
    }
}

impl Store<Vec<PartialLevel<u64, u64>>> for Cache<DB> {
    fn store(&mut self, partial_levels: &Vec<PartialLevel<u64, u64>>, key: u64) -> Result<Self::CacheEntryMeta, Self::Err> {
        use crate::partial_level::*;

        let conn = self.pool.get()?;

        diesel::delete(request_results::table)
            .filter(request_results::request_hash.eq(key as i64))
            .execute(&conn)?;

        for level in partial_levels {
            self.store(level, level.level_id)?;

            diesel::insert_into(request_results::table)
                .values((level.level_id, key))
                .execute(&conn)?;
        }

        let entry = Entry::new(key);

        upsert!(self, entry, level_list_meta::table, level_list_meta::request_hash);

        Ok(entry)
    }
}
