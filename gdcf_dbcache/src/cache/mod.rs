#[cfg(feature = "postgres")]
mod pg;

#[cfg(feature = "postgres")]
pub use self::pg::*;

#[cfg(feature = "mysql")]
mod mysql;

#[cfg(feature = "mysql")]
pub use self::mysql::*;

#[cfg(feature = "sqlite")]
mod sqlite;

#[cfg(feature = "sqlite")]
pub use self::sqlite::*;

use gdcf::api::request::LevelRequest;
use gdcf::api::request::LevelsRequest;
use gdcf::cache::Cache;
use gdcf::cache::CacheConfig;
use gdcf::cache::CachedObject;
use gdcf::model::song::NewgroundsSong;
use gdcf::model::Level;
use gdcf::model::PartialLevel;

use schema::song;
use schema::Cached;

use chrono::Duration;

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
    type Err = !;

    fn config(&self) -> &DatabaseCacheConfig {
        &self.config
    }

    fn lookup_partial_levels(
        &self,
        req: &LevelsRequest,
    ) -> Option<CachedObject<Vec<PartialLevel>>> {
        None
    }

    fn lookup_partial_level(&self, req: &LevelRequest) -> Option<CachedObject<PartialLevel>> {
        None
    }

    fn store_partial_level(&mut self, level: PartialLevel) {

    }

    fn lookup_level(&self, req: &LevelRequest) -> Option<CachedObject<Level>> {
        None
    }

    fn store_level(&mut self, level: Level) {

    }

    fn lookup_song(&self, newground_id: u64) -> Option<CachedObject<NewgroundsSong>> {
        CachedObject::retrieve(newground_id, &self.connection)
    }

    fn store_song(&mut self, song: NewgroundsSong) {
        CachedObject::store(song, &self.connection);
    }
}
