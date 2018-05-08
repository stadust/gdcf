use chrono::Duration;
use diesel::result::Error;
use gdcf::api::request::LevelRequest;
use gdcf::api::request::LevelsRequest;
use gdcf::cache::Cache;
use gdcf::cache::CacheConfig;
use gdcf::cache::CachedObject;
use gdcf::cache::Lookup;
use gdcf::error::CacheError;
use gdcf::model::Level;
use gdcf::model::PartialLevel;
use gdcf::model::song::NewgroundsSong;
use schema::Cached;
use schema::song;
#[cfg(feature = "mysql")]
pub use self::mysql::*;
#[cfg(feature = "postgres")]
pub use self::pg::*;
#[cfg(feature = "sqlite")]
pub use self::sqlite::*;

#[cfg(feature = "postgres")]
mod pg;

#[cfg(feature = "mysql")]
mod mysql;

#[cfg(feature = "sqlite")]
mod sqlite;

pub struct DatabaseCacheConfig {
    invalidate_after: Duration,
    url: &'static str,
}

impl CacheConfig for DatabaseCacheConfig {
    fn invalidate_after(&self) -> Duration {
        self.invalidate_after
    }
}

impl DatabaseCacheConfig {
    pub fn new(url: &'static str, invalidate_after: Duration) -> DatabaseCacheConfig {
        DatabaseCacheConfig {
            invalidate_after,
            url,
        }
    }
}

impl DatabaseCache {
    pub fn new(config: DatabaseCacheConfig) -> DatabaseCache {
        DatabaseCache {
            connection: connect(config.url),
            config,
        }
    }
}

impl Cache for DatabaseCache {
    type Config = DatabaseCacheConfig;
    type Err = Error;

    fn config(&self) -> &DatabaseCacheConfig {
        &self.config
    }

    fn lookup_partial_levels(&self, req: &LevelsRequest) -> Lookup<Vec<PartialLevel>, Self::Err> {
        Err(CacheError::CacheMiss)
    }

    fn store_partial_level(&mut self, level: PartialLevel) -> Result<(), CacheError<Self::Err>> {
        Err(CacheError::NoStore)
    }

    fn lookup_level(&self, req: &LevelRequest) -> Lookup<Level, Self::Err> {
        Err(CacheError::CacheMiss)
    }

    fn store_level(&mut self, level: Level) -> Result<(), CacheError<Self::Err>> {
        Err(CacheError::NoStore)
    }

    fn lookup_song(&self, newground_id: u64) -> Lookup<NewgroundsSong, Self::Err> {
        CachedObject::retrieve(newground_id, &self.connection)
            .map_err(|err| {
                if let Error::NotFound = err {
                    CacheError::CacheMiss
                } else {
                    CacheError::Custom(err)
                }
            })
    }

    fn store_song(&mut self, song: NewgroundsSong) -> Result<(), CacheError<Self::Err>> {
        CachedObject::store(song, &self.connection)
            .map_err(|err| CacheError::Custom(err))
    }
}
