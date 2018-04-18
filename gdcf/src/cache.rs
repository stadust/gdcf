use model::level::Level;

use std::time::{SystemTime, UNIX_EPOCH};
use model::level::PartialLevel;
use model::song::NewgroundSong;

pub struct CacheConfig {
    pub invalidate_after: u64
}

pub trait Cache
{
    fn config(&self) -> CacheConfig;

    fn lookup_partial_level(&self, lid: u64) -> Option<CachedObject<PartialLevel>>;
    fn store_partial_level(&mut self, level: PartialLevel);

    fn lookup_level(&self, lid: u64) -> Option<CachedObject<Level>>;
    fn store_level(&mut self, level: Level);

    fn lookup_song(&self, newground_id: u64) -> Option<CachedObject<NewgroundSong>>;
    fn store_song(&mut self, song: NewgroundSong);

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
    pub fn cached_at(&self) -> u64 { self.cached_at }
    pub fn extract(self) -> T { self.obj }
}

