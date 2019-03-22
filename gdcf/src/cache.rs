use crate::{error::CacheError, Secondary};
use chrono::NaiveDateTime;
use gdcf_model::{song::NewgroundsSong, user::Creator};

pub type Lookup<T, E> = Result<CachedObject<T>, E>;

pub trait Cache: Clone + Send + Sync + 'static {
    type Err: CacheError;

    fn lookup_song(&self, newground_id: u64) -> Result<NewgroundsSong, Self::Err>;
    fn lookup_creator(&self, user_id: u64) -> Result<Creator, Self::Err>;

    /// Stores an arbitrary [`Secondary`] in this [`Cache`]
    fn store_secondary(&mut self, obj: &Secondary) -> Result<(), Self::Err>;

    fn is_expired<T>(&self, object: &CachedObject<T>) -> bool;
}

pub trait CanCache<R: crate::api::request::Request>: Cache {
    fn lookup(&self, request: &R) -> Lookup<R::Result, Self::Err>;
    fn store(&mut self, object: &R::Result, request_hash: u64) -> Result<(), Self::Err>;
}

#[derive(Debug)]
pub struct CachedObject<T> {
    pub last_cached_at: NaiveDateTime,
    pub obj: T,
}

impl<T> CachedObject<T> {
    pub fn new(obj: T, last: NaiveDateTime) -> Self {
        CachedObject { last_cached_at: last, obj }
    }

    pub fn last_cached_at(&self) -> NaiveDateTime {
        self.last_cached_at
    }

    pub fn extract(self) -> T {
        self.obj
    }

    pub fn inner(&self) -> &T {
        &self.obj
    }

    pub fn map<U>(self, f: impl FnOnce(T) -> U) -> CachedObject<U> {
        CachedObject {
            last_cached_at: self.last_cached_at,
            obj: f(self.obj),
        }
    }

    pub fn try_map<R, E>(self, f: impl FnOnce(T) -> Result<R, E>) -> Result<CachedObject<R>, E> {
        Ok(CachedObject {
            last_cached_at: self.last_cached_at,
            obj: f(self.obj)?,
        })
    }
}
