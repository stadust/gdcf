use api::request::{user::UserRequest, LevelRequest, LevelsRequest};
use error::CacheError;
use model::{user::Creator, GDObject, Level, NewgroundsSong, PartialLevel, User};

use chrono::{DateTime, Duration, NaiveDateTime, Utc};
use failure::Fail;

pub type Lookup<T, E> = Result<CachedObject<T>, CacheError<E>>;

// TODO: more fine-grained cache-expiry control

pub trait CacheConfig {
    fn invalidate_after(&self) -> Duration;
}

pub trait Cache: Clone + Send + Sync {
    type Config: CacheConfig;
    type Err: Fail;

    fn config(&self) -> &Self::Config;

    fn lookup_partial_levels(&self, req: &LevelsRequest) -> Lookup<Vec<PartialLevel<u64, u64>>, Self::Err>;
    fn store_partial_levels(&mut self, req: &LevelsRequest, levels: &Vec<PartialLevel<u64, u64>>) -> Result<(), CacheError<Self::Err>>;

    fn lookup_level(&self, req: &LevelRequest) -> Lookup<Level<u64, u64>, Self::Err>;
    fn lookup_user(&self, req: &UserRequest) -> Lookup<User, Self::Err>;
    fn lookup_song(&self, newground_id: u64) -> Lookup<NewgroundsSong, Self::Err>;
    fn lookup_creator(&self, user_id: u64) -> Lookup<Creator, Self::Err>;

    /// Stores an arbitrary [`GDObject`] in this [`Cache`]
    fn store_object(&mut self, obj: &GDObject) -> Result<(), CacheError<Self::Err>>;

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

    pub fn inner(&self) -> &T {
        &self.obj
    }

    pub(crate) fn map<R, F>(self, f: F) -> CachedObject<R>
    where
        F: FnOnce(T) -> R,
    {
        let CachedObject {
            first_cached_at,
            last_cached_at,
            obj,
        } = self;

        CachedObject {
            first_cached_at,
            last_cached_at,
            obj: f(obj),
        }
    }

    pub(crate) fn try_map<R, F, E>(self, f: F) -> Result<CachedObject<R>, E>
    where
        F: FnOnce(T) -> Result<R, E>,
    {
        let CachedObject {
            first_cached_at,
            last_cached_at,
            obj,
        } = self;

        Ok(CachedObject {
            first_cached_at,
            last_cached_at,
            obj: f(obj)?,
        })
    }
}
