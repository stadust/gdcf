use chrono::{DateTime, Duration, NaiveDateTime, Utc};

use model::{PartialLevel, Level, NewgroundsSong};
use api::request::{LevelRequest, LevelsRequest};
use std::error::Error;

type Lookup<T> = Option<CachedObject<T>>;

pub trait CacheConfig {
    fn invalidate_after(&self) -> Duration;
}

pub trait Cache {
    type Config: CacheConfig;
    type Err: Error + 'static;

    fn config(&self) -> &Self::Config;

    fn lookup_partial_levels(&self, req: &LevelsRequest) -> Lookup<Vec<PartialLevel>>;
    fn lookup_partial_level(&self, req: &LevelRequest) -> Lookup<PartialLevel>;
    fn store_partial_level(&mut self, level: PartialLevel);

    fn lookup_level(&self, req: &LevelRequest) -> Lookup<Level>;
    fn store_level(&mut self, level: Level);

    fn lookup_song(&self, newground_id: u64) -> Lookup<NewgroundsSong>;
    fn store_song(&mut self, song: NewgroundsSong);

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
