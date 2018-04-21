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


use gdcf::cache::Cache;
use gdcf::cache::CacheConfig;
use gdcf::api::request::LevelsRequest;
use gdcf::cache::CachedObject;
use gdcf::model::PartialLevel;
use gdcf::api::request::LevelRequest;
use gdcf::model::Level;
use gdcf::model::song::NewgroundsSong;

use schema::song;
use schema::Cached;
use schema::song::Song;

impl DatabaseCache {
    pub fn new(url: &str) -> DatabaseCache {
        DatabaseCache {
            connection: connect(url)
        }
    }
}

impl Cache for DatabaseCache {
    fn config(&self) -> CacheConfig {
        CacheConfig {
            invalidate_after: 0
        }
    }

    fn lookup_partial_levels(&self, req: &LevelsRequest) -> Option<CachedObject<Vec<PartialLevel>>> {
        None
    }

    fn lookup_partial_level(&self, req: &LevelRequest) -> Option<CachedObject<PartialLevel>> {
        None
    }

    fn store_partial_level(&mut self, level: PartialLevel) {
        println!("Caching: {:?}", level);
    }

    fn lookup_level(&self, req: &LevelRequest) -> Option<CachedObject<Level>> {
        None
    }

    fn store_level(&mut self, level: Level) {
        println!("Caching: {:?}", level);
    }

    fn lookup_song(&self, newground_id: u64) -> Option<CachedObject<NewgroundsSong>> {
        Song::get(newground_id, &self.connection)
            .map(|Song(song)| CachedObject::new(song, 0))
    }

    fn store_song(&mut self, song: NewgroundsSong) {
        println!("Caching: {:?}", song);
        song::update_or_insert(song, &self.connection);
    }
}
