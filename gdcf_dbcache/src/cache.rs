#[cfg(feature = "pg")]
pub use crate::core::backend::pg::Pg;
#[cfg(feature = "sqlite")]
pub use crate::core::backend::sqlite::Sqlite;
use crate::{
    core::{
        backend::{Database, Error},
        query::{delete::Delete, insert::Insertable, select::Queryable, Insert, Query, Select},
    },
    schema::{
        level::{self, full_level, partial_level, partial_levels},
        song::{self, newgrounds_song},
        user::{self, creator, profile},
    },
};
use chrono::{Duration, Utc};
use gdcf::{
    api::request::{LevelRequest, LevelsRequest, UserRequest},
    cache::{Cache, CacheConfig, CachedObject, CanCache, Lookup},
    Secondary,
};
use gdcf_model::{
    level::{Level, PartialLevel},
    song::NewgroundsSong,
    user::{Creator, User},
};
use log::{debug, info};
use seahash::SeaHasher;
use std::hash::{Hash, Hasher};

#[derive(Debug, Clone)]
pub struct DatabaseCacheConfig<DB: Database> {
    backend: DB,
    invalidate_after: Duration,
}

impl<DB: Database> DatabaseCacheConfig<DB> {
    pub fn new(backend: DB) -> DatabaseCacheConfig<DB> {
        DatabaseCacheConfig {
            backend,
            invalidate_after: Duration::zero(),
        }
    }

    pub fn invalidate_after(&mut self, duration: Duration) {
        self.invalidate_after = duration;
    }
}

#[cfg(feature = "pg")]
impl DatabaseCacheConfig<Pg> {
    pub fn postgres_config(url: &str) -> Result<DatabaseCacheConfig<Pg>, Error<Pg>> {
        use r2d2_postgres::TlsMode;

        Ok(DatabaseCacheConfig::new(Pg::new(url, TlsMode::None)?))
    }
}

#[cfg(feature = "sqlite")]
use std::path::Path;

#[cfg(feature = "sqlite")]
impl DatabaseCacheConfig<Sqlite> {
    pub fn sqlite_memory_config() -> Result<DatabaseCacheConfig<Sqlite>, Error<Sqlite>> {
        Ok(DatabaseCacheConfig::new(Sqlite::memory()?))
    }

    pub fn sqlite_config<P: AsRef<Path>>(path: P) -> Result<DatabaseCacheConfig<Sqlite>, Error<Sqlite>> {
        Ok(DatabaseCacheConfig::new(Sqlite::path(path)?))
    }
}

impl<DB: Database + 'static> CacheConfig for DatabaseCacheConfig<DB> {
    fn invalidate_after(&self) -> Duration {
        self.invalidate_after
    }
}

#[derive(Debug, Clone)]
pub struct DatabaseCache<DB: Database> {
    config: DatabaseCacheConfig<DB>,
}

impl<DB: Database> DatabaseCache<DB> {
    pub fn new(config: DatabaseCacheConfig<DB>) -> DatabaseCache<DB> {
        DatabaseCache { config }
    }
}

macro_rules! cache {
    ($feature: expr, $backend: ty) => {
        #[cfg(feature = $feature)]
        impl DatabaseCache<$backend> {
            pub fn initialize(&self) -> Result<(), Error<$backend>> {
                info!("Intializing {}-backed database cache!", stringify!($backend));

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
                level::full_level::create().ignore_if_exists().execute(&self.config.backend)?;
                user::creator::create().ignore_if_exists().execute(&self.config.backend)?;
                user::profile::create().ignore_if_exists().execute(&self.config.backend)?;

                Ok(())
            }
        }

        #[cfg(feature = $feature)]
        impl DatabaseCache<$backend> {
            /// Retrieves every partial level stored in the database
            pub fn all_partial_levels(&self) -> Result<Vec<PartialLevel<u64, u64>>, <Self as Cache>::Err> {
                self.config.backend.query(&partial_level::table.select())
            }
        }

        #[cfg(feature = $feature)]
        impl CanCache<LevelRequest> for DatabaseCache<$backend> {
            fn store(&mut self, level: &Level<u64, u64>, _: u64) -> Result<(), <Self as Cache>::Err> {
                debug!("Storing level {}", level);

                let ts = Utc::now().naive_utc();

                self.store_partial_level(&level.base)?;

                level
                    .insert()
                    .with(full_level::level_id.set(&level.base.level_id))
                    .with(full_level::last_cached_at.set(&ts))
                    .on_conflict_update(vec![&full_level::level_id])
                    .execute(&self.config.backend)
            }

            fn lookup(&self, req: &LevelRequest) -> Lookup<Level<u64, u64>, Self::Err> {
                let select = Level::select_from(full_level::table).filter(full_level::level_id.eq(req.level_id));

                self.config.backend.query_one(&select)
            }
        }

        #[cfg(feature = $feature)]
        impl CanCache<LevelsRequest> for DatabaseCache<$backend> {
            fn lookup(&self, req: &LevelsRequest) -> Lookup<Vec<PartialLevel<u64, u64>>, Self::Err> {
                debug!("Performing cache lookup for request {}", req);

                let h = self.hash(req);

                let select = Select::new(
                    partial_levels::cached_at::table,
                    vec![
                        partial_levels::cached_at::first_cached_at,
                        partial_levels::cached_at::last_cached_at,
                    ],
                )
                .filter(partial_levels::cached_at::request_hash.eq(h));

                let row = self.config.backend.query_one_row(&select)?;

                let first_cached_at = row.get(0).unwrap()?;

                let last_cached_at = row.get(1).unwrap()?;

                let select = Select::new(partial_levels::table, Vec::new())
                    .select(partial_level::table.fields())
                    .filter(partial_levels::request_hash.eq(h))
                    .join(
                        partial_level::table,
                        partial_levels::level_id.same_as(partial_level::level_id),
                    );

                let levels: Vec<PartialLevel<u64, u64>> = self.config.backend.query(&select)?;

                Ok(CachedObject::new(levels, first_cached_at, last_cached_at))
            }

            fn store(&mut self, levels: &Vec<PartialLevel<u64, u64>>, request_hash: u64) -> Result<(), <Self as Cache>::Err> {
                let ts = Utc::now().naive_utc();

                // TODO: wrap this into a transaction if I can figure out how to implement them
                Delete::new(&partial_levels::table)
                    .if_met(partial_levels::request_hash.eq(request_hash))
                    .execute(&self.config.backend)?;

                Insert::new(partial_levels::cached_at::table, Vec::new())
                    .with(partial_levels::cached_at::last_cached_at.set(&ts))
                    .with(partial_levels::cached_at::request_hash.set(&request_hash))
                    .on_conflict_update(vec![&partial_levels::cached_at::request_hash])
                    .execute(&self.config.backend)?;

                for level in levels {
                    self.store_partial_level(&level)?;

                    Insert::new(partial_levels::table, Vec::new())
                        .with(partial_levels::request_hash.set(&request_hash))
                        .with(partial_levels::level_id.set(&level.level_id))
                        .execute(&self.config.backend)?;
                }

                Ok(())
            }
        }

        #[cfg(feature = $feature)]
        impl CanCache<UserRequest> for DatabaseCache<$backend> {
            fn lookup(&self, req: &UserRequest) -> Lookup<User, Self::Err> {
                let select = User::select_from(profile::table).filter(profile::account_id.eq(req.user));

                self.config.backend.query_one(&select)
            }

            fn store(&mut self, user: &User, _: u64) -> Result<(), <Self as Cache>::Err> {
                debug!("Storing user {}", user);

                let ts = Utc::now().naive_utc();

                user.insert()
                    .with(profile::last_cached_at.set(&ts))
                    .on_conflict_update(vec![&profile::user_id])
                    .execute(&self.config.backend)
            }
        }
    };
}

cache!("pg", Pg);
cache!("sqlite", Sqlite);

use crate::core::AsSql;
use chrono::NaiveDateTime;

impl<DB: Database + Send + Sync + Clone> DatabaseCache<DB>
where
    NewgroundsSong: Queryable<DB> + Insertable<DB>,
    PartialLevel<u64, u64>: Insertable<DB>,
    Creator: Insertable<DB>,
    NaiveDateTime: AsSql<DB>,
    for<'a> Insert<'a, DB>: Query<DB>,
{
    fn store_song(&self, song: &NewgroundsSong) -> Result<(), Error<DB>> {
        debug!("Storing song {}", song);

        let ts = Utc::now().naive_utc();

        song.insert()
            .with(newgrounds_song::last_cached_at.set(&ts))
            .on_conflict_update(vec![&newgrounds_song::song_id])
            .execute(&self.config.backend)
    }

    fn store_creator(&self, creator: &Creator) -> Result<(), Error<DB>> {
        debug!("Storing creator {}", creator);

        let ts = Utc::now().naive_utc();

        creator
            .insert()
            .with(creator::last_cached_at.set(&ts))
            .on_conflict_update(vec![&creator::user_id])
            .execute(&self.config.backend)
    }

    fn store_partial_level(&self, level: &PartialLevel<u64, u64>) -> Result<(), Error<DB>> {
        debug!("Storing partial level {}", level);

        let ts = Utc::now().naive_utc();

        level
            .insert()
            .with(partial_level::last_cached_at.set(&ts))
            .on_conflict_update(vec![&partial_level::level_id])
            .execute(&self.config.backend)
    }
}

use crate::core::{
    query::condition::{And, Condition, EqValue},
    FromSql, QueryPart,
};

impl<DB: Database + Sync + Send + Clone + 'static> Cache for DatabaseCache<DB>
where
    NewgroundsSong: Queryable<DB> + Insertable<DB>,
    PartialLevel<u64, u64>: Queryable<DB> + Insertable<DB>,
    Creator: Queryable<DB> + Insertable<DB>,
    NaiveDateTime: AsSql<DB> + FromSql<DB>,
    for<'a> Insert<'a, DB>: Query<DB>,
    Select<DB>: QueryPart<DB>,
    EqValue<DB>: Condition<DB>,
    And<DB>: QueryPart<DB>,
    u64: AsSql<DB>,
{
    // we cannot turn this into an impl generic over DB: Database
    // because it would require us to add explicit trait bounds for all the
    // structs used for query building, which in turn would require us to add a
    // lifetime to the impl. This would force all structs used internally to be bound to
    // that lifetime, although they only live for the function they're used in.
    // Since that obviously results in compiler errors, we cant do that.
    // (and we also cant add the lifetimes directly to the functions, because trait)

    type Config = DatabaseCacheConfig<DB>;
    type Err = Error<DB>;

    fn config(&self) -> &DatabaseCacheConfig<DB> {
        &self.config
    }

    fn lookup_song(&self, newground_id: u64) -> Lookup<NewgroundsSong, Error<DB>> {
        let select = NewgroundsSong::select_from(newgrounds_song::table).filter(newgrounds_song::song_id.eq(newground_id));

        self.config.backend.query_one(&select)
    }

    fn lookup_creator(&self, user_id: u64) -> Lookup<Creator, Error<DB>> {
        let select = Creator::select_from(creator::table).filter(creator::user_id.eq(user_id));

        self.config.backend.query_one(&select)
    }

    fn store_secondary(&mut self, obj: &Secondary) -> Result<(), Error<DB>> {
        match obj {
            Secondary::NewgroundsSong(song) => self.store_song(song),
            Secondary::Creator(creator) => self.store_creator(creator),
        }
    }

    fn hash<H: Hash>(&self, h: H) -> u64 {
        let mut hasher = SeaHasher::default();
        h.hash(&mut hasher);
        hasher.finish()
    }
}
