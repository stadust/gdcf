mod creator;
mod song;

// diesel devs refuse to make their macros work with the new rust 2018 import mechanics, so this
// shit is necessary

#[macro_use]
extern crate diesel;
#[macro_use]
extern crate diesel_migrations;

use crate::song::newgrounds_song;
use diesel::{r2d2::ConnectionManager, Connection, Insertable, RunQueryDsl};
use failure::Fail;
use gdcf::{cache::CachedObject, error::CacheError, Secondary};
use gdcf_model::{song::NewgroundsSong, user::Creator};
use r2d2::Pool;

pub struct Cache<T: Connection + 'static>(Pool<ConnectionManager<T>>);

impl<T: Connection + 'static> Clone for Cache<T> {
    fn clone(&self) -> Self {
        Cache(self.0.clone())
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
    use diesel::{r2d2::ConnectionManager, sqlite::SqliteConnection};
    use r2d2::Pool;
    use std::path::Path;

    embed_migrations!("migrations/sqlite");

    impl Cache<SqliteConnection> {
        pub fn in_memory() -> Result<Self, r2d2::Error> {
            Pool::new(ConnectionManager::new(":memory:")).map(Self)
        }

        pub fn sqlite(path: String) -> Result<Self, r2d2::Error> {
            Pool::new(ConnectionManager::new(path)).map(Self)
        }

        pub fn initialize(&self) -> Result<(), diesel_migrations::RunMigrationsError> {
            embedded_migrations::run(&self.0.get().unwrap())
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
            Error::CacheMiss => true,
            _ => false,
        }
    }
}

impl gdcf::cache::Cache for Cache<DB> {
    type Err = Error;

    fn lookup_song(&self, newground_id: u64) -> Result<NewgroundsSong, Self::Err> {
        unimplemented!()
    }

    fn lookup_creator(&self, user_id: u64) -> Result<Creator, Self::Err> {
        unimplemented!()
    }

    fn store_secondary(&mut self, obj: &Secondary) -> Result<(), Self::Err> {
        let connection = self.0.get()?;

        match obj {
            Secondary::Creator(creator) => creator.insert_into(crate::creator::creator::table).execute(&connection)?,
            Secondary::NewgroundsSong(song) => song.insert_into(newgrounds_song::table).execute(&connection)?,
        };

        Ok(())
    }

    fn is_expired<U>(&self, object: &CachedObject<U>) -> bool {
        unimplemented!()
    }
}
