#![recursion_limit = "128"]
#![deny(unused_must_use)]
#![deny(unused_imports)]
#[macro_use]
mod meta;
#[macro_use]
mod macros;
mod creator;
mod level;
mod partial_level;
mod profile;
mod song;
mod wrap;

// diesel devs refuse to make their macros work with the new rust 2018 import mechanics, so this
// shit is necessary

#[macro_use]
extern crate diesel;
#[macro_use]
extern crate diesel_migrations;

use crate::{meta::DatabaseEntry, wrap::Wrapped};
use chrono::{DateTime, Duration, Utc};
use diesel::{query_dsl::QueryDsl, r2d2::ConnectionManager, ExpressionMethods, RunQueryDsl};
use failure::Fail;
use gdcf::{
    cache::{CacheEntry, Lookup, Store},
    error::CacheError,
};
use gdcf_model::level::PartialLevel;
use log::{debug, warn};
use r2d2::Pool;

pub use crate::meta::Entry;

// this means we cannot enable two features at once. Since diesel doesn't allow writing database
// agnostic code, the alternative to this is wrapping everything in macros (like we used to do in
// gdcf_dbcache). That's a crappy alternative. We dont do that there
#[cfg(feature = "pg")]
use diesel::pg::PgConnection;

#[cfg(feature = "sqlite")]
use diesel::sqlite::SqliteConnection;

pub struct Cache {
    #[cfg(feature = "pg")]
    pool: Pool<ConnectionManager<PgConnection>>,
    #[cfg(feature = "sqlite")]
    pool: Pool<ConnectionManager<SqliteConnection>>,
    expire_after: Duration,
}

impl Cache {
    fn entry(&self, db_entry: DatabaseEntry) -> Entry {
        let now = Utc::now();
        let then = DateTime::<Utc>::from_utc(db_entry.cached_at, Utc);
        let expired = now - then > self.expire_after;

        Entry {
            expired,
            key: db_entry.key as u64,
            cached_at: db_entry.cached_at,
            absent: db_entry.absent,
        }
    }
}

impl Clone for Cache {
    fn clone(&self) -> Self {
        Cache {
            pool: self.pool.clone(),
            expire_after: self.expire_after,
        }
    }
}

#[cfg(feature = "pg")]
mod postgres {
    use super::Cache;
    use chrono::Duration;
    use diesel::r2d2::ConnectionManager;
    use r2d2::Pool;

    embed_migrations!("migrations/postgres");

    impl Cache {
        pub fn postgres(database_url: impl Into<String>) -> Result<Self, r2d2::Error> {
            Ok(Cache {
                pool: Pool::new(ConnectionManager::new(database_url.into()))?,
                expire_after: Duration::minutes(60),
            })
        }

        pub fn initialize(&self) -> Result<(), diesel_migrations::RunMigrationsError> {
            embedded_migrations::run(&self.pool.get().unwrap())
        }
    }
}

#[cfg(feature = "sqlite")]
mod sqlite {
    use super::Cache;
    use chrono::Duration;
    use diesel::r2d2::ConnectionManager;
    use r2d2::Pool;

    embed_migrations!("migrations/sqlite");

    impl Cache {
        pub fn in_memory() -> Result<Self, r2d2::Error> {
            Ok(Self {
                pool: Pool::new(ConnectionManager::new(":memory:"))?,
                expire_after: Duration::seconds(60),
            })
        }

        pub fn sqlite(path: impl Into<String>) -> Result<Self, r2d2::Error> {
            Ok(Self {
                pool: Pool::new(ConnectionManager::new(path.into()))?,
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

impl CacheError for Error {}

impl gdcf::cache::Cache for Cache {
    type CacheEntryMeta = Entry;
    type Err = Error;
}

// TODO: in the future we can probably make these macro-generated as well, but for now we only have
// one of them, so its fine

impl Lookup<Vec<PartialLevel<Option<u64>, u64>>> for Cache {
    fn lookup(&self, key: u64) -> Result<CacheEntry<Vec<PartialLevel<Option<u64>, u64>>, Entry>, Self::Err> {
        use crate::partial_level::*;
        use diesel::JoinOnDsl;

        let connection = self.pool.get()?;

        let entry = handle_missing!(level_list_meta::table
            .filter(level_list_meta::request_hash.eq(key as i64))
            .get_result(&connection));

        let entry = self.entry(entry);

        if entry.absent {
            return Ok(CacheEntry::MarkedAbsent(entry))
        }

        let levels: Vec<_> = handle_missing!(partial_level::table
            .inner_join(level_request_results::table.on(partial_level::level_id.eq(level_request_results::level_id)))
            .filter(level_request_results::request_hash.eq(key as i64))
            .select(partial_level::all_columns)
            .load(&connection))
        .into_iter()
        .map(|row: Wrapped<_>| row.0)
        .collect();

        if levels.is_empty() {
            Ok(CacheEntry::DeducedAbsent)
        } else {
            Ok(CacheEntry::new(levels, entry))
        }
    }
}

impl Store<Vec<PartialLevel<Option<u64>, u64>>> for Cache {
    fn mark_absent(&mut self, key: u64) -> Result<Entry, Self::Err> {
        use crate::partial_level::*;

        warn!("Marking results of LevelsRequest with key {} as absent!", key);

        let entry = Entry::absent(key);

        update_entry!(self, entry, level_list_meta::table, level_list_meta::request_hash);

        Ok(entry)
    }

    fn store(&mut self, partial_levels: &Vec<PartialLevel<Option<u64>, u64>>, key: u64) -> Result<Self::CacheEntryMeta, Self::Err> {
        use crate::partial_level::*;

        debug!("Storing result of LevelsRequest with key {}", key);

        let conn = self.pool.get()?;

        diesel::delete(level_request_results::table)
            .filter(level_request_results::request_hash.eq(key as i64))
            .execute(&conn)?;

        for level in partial_levels {
            self.store(level, level.level_id)?;

            diesel::insert_into(level_request_results::table)
                .values((level.level_id, key))
                .execute(&conn)?;
        }

        let entry = Entry::new(key);

        update_entry!(self, entry, level_list_meta::table, level_list_meta::request_hash);

        Ok(entry)
    }
}
