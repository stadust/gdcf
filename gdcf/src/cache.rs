use model::level::Level;

use api::request::LevelRequest;
use api::request::LevelsRequest;
use chrono::DateTime;
use chrono::Duration;
use chrono::NaiveDateTime;
use chrono::NaiveTime;
use chrono::Utc;
use model::level::PartialLevel;
use model::song::NewgroundsSong;
use std::time::{SystemTime, UNIX_EPOCH};

pub trait CacheConfig {
    fn invalidate_after(&self) -> Duration;
}

pub trait Cache {
    type Config: CacheConfig;

    fn config(&self) -> &Self::Config;

    fn lookup_partial_levels(&self, req: &LevelsRequest) -> Option<CachedObject<Vec<PartialLevel>>>;
    fn lookup_partial_level(&self, req: &LevelRequest) -> Option<CachedObject<PartialLevel>>;
    fn store_partial_level(&mut self, level: PartialLevel);

    fn lookup_level(&self, req: &LevelRequest) -> Option<CachedObject<Level>>;
    fn store_level(&mut self, level: Level);

    fn lookup_song(&self, newground_id: u64) -> Option<CachedObject<NewgroundsSong>>;
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
