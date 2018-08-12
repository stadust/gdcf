use chrono::{Duration, Utc};
use core::backend::Database;
use core::backend::Error;
#[cfg(feature = "pg")]
use core::backend::pg::Pg;
use core::query::insert::Insertable;
use core::query::Query;
use core::query::select::Queryable;
use gdcf::api::request::LevelRequest;
use gdcf::api::request::LevelsRequest;
use gdcf::cache::Cache;
use gdcf::cache::CacheConfig;
use gdcf::cache::Lookup;
use gdcf::error::CacheError;
use gdcf::model::Level;
use gdcf::model::NewgroundsSong;
use gdcf::model::PartialLevel;
use schema::level::{self, partial_level};
use schema::song::{self, newgrounds_song};
use util;

#[derive(Debug)]
pub struct DatabaseCacheConfig<DB: Database> {
    backend: DB,
}

impl<DB: Database> DatabaseCacheConfig<DB> {
    pub fn new(backend: DB) -> DatabaseCacheConfig<DB> {
        DatabaseCacheConfig {
            backend
        }
    }
}

#[cfg(feature = "pg")]
impl DatabaseCacheConfig<Pg> {
    pub fn postgres_config(url: &str) -> DatabaseCacheConfig<Pg> {
        use postgres::TlsMode;

        DatabaseCacheConfig::new(Pg::new(url, TlsMode::None))
    }
}

impl<DB: Database + 'static> CacheConfig for DatabaseCacheConfig<DB> {
    // TODO: figure out a way to specify this
    fn invalidate_after(&self) -> Duration {
        unimplemented!()
    }
}

#[derive(Debug)]
pub struct DatabaseCache<DB: Database> {
    config: DatabaseCacheConfig<DB>
}

impl<DB: Database> DatabaseCache<DB> {
    pub fn new(config: DatabaseCacheConfig<DB>) -> DatabaseCache<DB> {
        DatabaseCache {
            config
        }
    }
}

#[cfg(feature = "pg")]
impl DatabaseCache<Pg> {
    pub fn initialize(&self) -> Result<(), Error<Pg>> {
        song::newgrounds_song::create()
            .ignore_if_exists()
            .execute(&self.config.backend)?;
        level::partial_level::create()
            .ignore_if_exists()
            .execute(&self.config.backend)?;
        level::partial_levels::create()
            .ignore_if_exists()
            .execute(&self.config.backend);

        Ok(())
    }
}

// TODO: turn cache impl into a macro
#[cfg(feature = "pg")]
impl Cache for DatabaseCache<Pg>
{
    // we cannot turn this into an impl generic over DB: Database
    // because it would require us to add explicit trait bounds for all the
    // structs used for query building, which in turn would require us to add a
    // lifetime to the impl. This would force all structs used internally to be bound to
    // that lifetime, although they only live for the function they're used in.
    // Since that obviously results in compiler errors, we cant do that.
    // (and we also cant add the lifetimes directly to the functions, because trait)

    type Config = DatabaseCacheConfig<Pg>;
    type Err = Error<Pg>;

    fn config(&self) -> &DatabaseCacheConfig<Pg> {
        &self.config
    }

    fn lookup_partial_levels(&self, req: &LevelsRequest) -> Lookup<Vec<PartialLevel>, Self::Err> {
        let select = PartialLevel::select_from(&partial_level::table);

        let h = util::hash(req);

        /*self.config.backend.query_one(&select)
            .map_err(Into::into)*/
        Err(CacheError::CacheMiss)
    }

    fn store_partial_level(&mut self, level: PartialLevel) -> Result<(), CacheError<Self::Err>> {
        let ts = Utc::now().naive_utc();

        level.insert()
            .with(partial_level::last_cached_at.set(&ts))
            .on_conflict_update(vec![&partial_level::level_id])
            .execute(&self.config.backend)
            .map_err(CacheError::Custom)
    }

    fn lookup_level(&self, req: &LevelRequest) -> Lookup<Level, Self::Err> {
        Err(CacheError::CacheMiss)
    }

    fn store_level(&mut self, level: Level) -> Result<(), CacheError<Self::Err>> {
        self.store_partial_level(level.base)?;

        Err(CacheError::NoStore)
    }

    fn lookup_song(&self, newground_id: u64) -> Lookup<NewgroundsSong, Self::Err> {
        let select = NewgroundsSong::select_from(&newgrounds_song::table)
            .filter(newgrounds_song::song_id.eq(newground_id));

        self.config.backend.query_one(&select)
            .map_err(Into::into)  // for some reason I can use the question mark operator?????
    }

    fn store_song(&mut self, song: NewgroundsSong) -> Result<(), CacheError<Self::Err>> {
        let ts = Utc::now().naive_utc();

        song.insert()
            .with(newgrounds_song::last_cached_at.set(&ts))
            .on_conflict_update(vec![&newgrounds_song::song_id])
            .execute(&self.config.backend)
            .map_err(CacheError::Custom)
    }
}

impl<DB: Database> Into<CacheError<Error<DB>>> for Error<DB> {
    fn into(self) -> CacheError<Error<DB>> {
        match self {
            Error::NoResult => CacheError::CacheMiss,
            err => CacheError::Custom(err)
        }
    }
}