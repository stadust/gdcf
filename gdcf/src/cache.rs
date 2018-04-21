use model::level::Level;

use std::time::{SystemTime, UNIX_EPOCH};
use model::level::PartialLevel;
use model::song::NewgroundsSong;
use api::request::LevelRequest;
use api::request::LevelsRequest;

pub struct CacheConfig {
    pub invalidate_after: u64
}

pub trait Cache
{
    fn config(&self) -> CacheConfig;

    fn lookup_partial_levels(&self, req: &LevelsRequest) -> Option<CachedObject<Vec<PartialLevel>>>;
    fn lookup_partial_level(&self, req: &LevelRequest) -> Option<CachedObject<PartialLevel>>;
    fn store_partial_level(&mut self, level: PartialLevel);

    fn lookup_level(&self, req: &LevelRequest) -> Option<CachedObject<Level>>;
    fn store_level(&mut self, level: Level);

    fn lookup_song(&self, newground_id: u64) -> Option<CachedObject<NewgroundsSong>>;
    fn store_song(&mut self, song: NewgroundsSong);

    fn is_expired<T>(&self, obj: &CachedObject<T>) -> bool {
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs() - obj.cached_at > self.config().invalidate_after
    }
}

#[derive(Debug)]
pub struct CachedObject<T> {
    cached_at: u64,
    obj: T,
}

impl<T> CachedObject<T> {
    pub fn new(obj: T, cached_at: u64) -> Self { CachedObject { cached_at, obj } }
    pub fn cached_at(&self) -> u64 { self.cached_at }
    pub fn extract(self) -> T { self.obj }
}
