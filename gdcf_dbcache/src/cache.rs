use chrono::{Duration, Utc};
use chrono::NaiveDateTime;
use core::AsSql;
use core::backend::Database;
use core::backend::Error;
use core::query::Insert;
use core::query::insert::Insertable;
use core::query::Query;
use gdcf::api::request::LevelRequest;
use gdcf::api::request::LevelsRequest;
use gdcf::cache::Cache;
use gdcf::cache::CacheConfig;
use gdcf::cache::Lookup;
use gdcf::error::CacheError;
use gdcf::model::Level;
use gdcf::model::NewgroundsSong;
use gdcf::model::PartialLevel;
use schema::song::newgrounds_song;

pub struct DatabaseCacheConfig<DB: Database> {
    backend: DB
}

impl<DB: Database> DatabaseCacheConfig<DB> {
    pub fn new(backend: DB) -> DatabaseCacheConfig<DB> {
        DatabaseCacheConfig {
            backend
        }
    }
}

impl<DB: Database + 'static> CacheConfig for DatabaseCacheConfig<DB> {
    fn invalidate_after(&self) -> Duration {
        unimplemented!()
    }
}

struct DatabaseCache<DB: Database> {
    config: DatabaseCacheConfig<DB>
}

impl<'a, DB: Database + 'static> Cache for DatabaseCache<DB>
    where
        //Insert<'_, DB>: Query<'a, DB>,
        NewgroundsSong: Insertable<DB>,
        NaiveDateTime: AsSql<DB>
{
    type Config = DatabaseCacheConfig<DB>;
    type Err = Error<DB>;

    fn config(&self) -> &DatabaseCacheConfig<DB> {
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

    fn store_level(&mut self, level: Level) -> Result<(), CacheError<<Self as Cache>::Err>> {
        Err(CacheError::NoStore)
    }

    fn lookup_song(&self, newground_id: u64) -> Lookup<NewgroundsSong, Self::Err> {
        Err(CacheError::CacheMiss)
    }

    fn store_song(&mut self, song: NewgroundsSong) -> Result<(), CacheError<Self::Err>> {
        let ts = Utc::now().naive_utc();

        song.insert()
            .with(newgrounds_song::last_cached_at.set(&ts))
            .on_conflict_update()
            .execute(&self.config.backend)
    }
}