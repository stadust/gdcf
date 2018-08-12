use chrono::{Duration, Utc};
use core::backend::Database;
use core::backend::Error;
#[cfg(feature = "pg")]
use core::backend::pg::Pg;
use core::query::delete::Delete;
use core::query::Insert;
use core::query::insert::Insertable;
use core::query::Query;
use core::query::select::Queryable;
use gdcf::api::request::LevelRequest;
use gdcf::api::request::LevelsRequest;
use gdcf::cache::Cache;
use gdcf::cache::CacheConfig;
use gdcf::cache::Lookup;
use gdcf::error::CacheError;
use gdcf::model::GDObject;
use gdcf::model::Level;
use gdcf::model::NewgroundsSong;
use gdcf::model::PartialLevel;
use schema::level::{self, partial_level, partial_levels};
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
            .execute(&self.config.backend)?;
        level::partial_levels::cached_at::create()
            .ignore_if_exists()
            .execute(&self.config.backend)?;

        Ok(())
    }
}

#[cfg(feature = "pg")]
impl DatabaseCache<Pg> {
    fn store_partial_level(&self, level: PartialLevel) -> Result<(), CacheError<<Self as Cache>::Err>> {
        let ts = Utc::now().naive_utc();

        level.insert()
            .with(partial_level::last_cached_at.set(&ts))
            .on_conflict_update(vec![&partial_level::level_id])
            .execute(&self.config.backend)
            .map_err(convert_error)
    }

    fn store_song(&self, song: NewgroundsSong) -> Result<(), CacheError<<Self as Cache>::Err>> {
        let ts = Utc::now().naive_utc();

        song.insert()
            .with(newgrounds_song::last_cached_at.set(&ts))
            .on_conflict_update(vec![&newgrounds_song::song_id])
            .execute(&self.config.backend)
            .map_err(convert_error)
    }

    fn store_level(&self, level: Level) -> Result<(), CacheError<<Self as Cache>::Err>> {
        self.store_partial_level(level.base)?;

        Err(CacheError::NoStore)
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

    fn store_partial_levels(&mut self, req: &LevelsRequest, levels: Vec<PartialLevel>) -> Result<(), CacheError<Self::Err>> {
        let h = util::hash(req);
        let ts = Utc::now().naive_utc();

        // TODO: wrap this into a transaction if I can figure out how to implement them
        Delete::new(&partial_levels::table)
            .if_met(partial_levels::request_hash.eq(h))
            .execute(&self.config.backend)
            .map_err(convert_error)?;

        Insert::new(&partial_levels::cached_at::table, Vec::new())
            .with(partial_levels::cached_at::last_cached_at.set(&ts))
            .with(partial_levels::cached_at::request_hash.set(&h))
            .on_conflict_update(vec![&partial_levels::cached_at::request_hash])
            .execute(&self.config.backend)
            .map_err(convert_error)?;

        for level in levels {
            Insert::new(&partial_levels::table, Vec::new())
                .with(partial_levels::request_hash.set(&h))
                .with(partial_levels::level_id.set(&level.level_id))
                .execute(&self.config.backend)
                .map_err(convert_error)?;
        }

        Ok(())
    }

    fn lookup_level(&self, req: &LevelRequest) -> Lookup<Level, Self::Err> {
        Err(CacheError::CacheMiss)
    }

    fn lookup_song(&self, newground_id: u64) -> Lookup<NewgroundsSong, Self::Err> {
        let select = NewgroundsSong::select_from(&newgrounds_song::table)
            .filter(newgrounds_song::song_id.eq(newground_id));

        self.config.backend.query_one(&select)
            .map_err(convert_error)  // for some reason I can use the question mark operator?????
    }

    fn store_object(&self, obj: GDObject) -> Result<(), CacheError<<Self as Cache>::Err>> {
        match obj {
            GDObject::PartialLevel(lvl) => self.store_partial_level(lvl),
            GDObject::NewgroundsSong(song) => self.store_song(song),
            GDObject::Level(lvl) => self.store_level(lvl)
        }
    }
}

fn convert_error<DB: Database>(db_error: Error<DB>) -> CacheError<Error<DB>> {
    match db_error {
        Error::NoResult => CacheError::CacheMiss,
        _ => CacheError::Custom(db_error)
    }
}