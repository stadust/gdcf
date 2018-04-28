use chrono::{DateTime, Duration, NaiveDateTime, Utc};

use model::{PartialLevel, Level, NewgroundsSong};
use api::request::{LevelRequest, LevelsRequest};
use std::error::Error;
use error::CacheError;
use std::result;

pub type Lookup<T, E> = result::Result<CachedObject<T>, CacheError<E>>;

pub trait CacheConfig {
    fn invalidate_after(&self) -> Duration;
}

pub trait Cache {
    type Config: CacheConfig;
    type Err: Error + 'static;

    fn config(&self) -> &Self::Config;

    fn lookup_partial_levels(&self, req: &LevelsRequest) -> Lookup<Vec<PartialLevel>, Self::Err>;
    fn store_partial_level(&mut self, level: PartialLevel) -> Result<(), CacheError<Self::Err>>;

    fn lookup_level(&self, req: &LevelRequest) -> Lookup<Level, Self::Err>;
    fn store_level(&mut self, level: Level) -> Result<(), CacheError<Self::Err>>;

    fn lookup_song(&self, newground_id: u64) -> Lookup<NewgroundsSong, Self::Err>;
    fn store_song(&mut self, song: NewgroundsSong) -> Result<(), CacheError<Self::Err>>;

    fn is_expired<T>(&self, obj: &CachedObject<T>) -> bool {
        let now = Utc::now();
        let then = DateTime::<Utc>::from_utc(obj.last_cached_at(), Utc);

        now - then > self.config().invalidate_after()
    }
}

#[derive(Debug)]
pub struct CachedObject<T> {
    first_cached_at: NaiveDateTime,
    last_cached_at: NaiveDateTime,
    obj: T,
}

impl<T> CachedObject<T> {
    pub fn new(obj: T, first: NaiveDateTime, last: NaiveDateTime) -> Self {
        CachedObject {
            first_cached_at: first,
            last_cached_at: last,
            obj,
        }
    }

    pub fn last_cached_at(&self) -> NaiveDateTime {
        self.last_cached_at
    }

    pub fn first_cached_at(&self) -> NaiveDateTime {
        self.first_cached_at
    }

    pub fn extract(self) -> T {
        self.obj
    }
}
