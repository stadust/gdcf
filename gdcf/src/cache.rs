use crate::{api::request::Request, error::CacheError, Secondary};
use chrono::NaiveDateTime;
use gdcf_model::{song::NewgroundsSong, user::Creator};

pub type Lookup2<T, E> = Result<CachedObject<T>, E>;

pub trait Cache: Clone + Send + Sync + 'static {
    type Err: CacheError;
    type CacheEntryMeta;

    fn is_expired<T>(&self, object: &CachedObject<T>) -> bool;
}

pub trait Lookup<Obj>: Cache {
    type Key;

    fn lookup(&self, id: &Self::Key) -> Lookup2<Obj, Self::Err>; //Result<(Obj, Self::CacheEntryMeta), Self::Err>;
}

pub trait Store<Obj>: Cache {
    type Key;

    fn store(&mut self, obj: &Obj, key: Self::Key) -> Result<(), Self::Err>;
}

#[derive(Debug)]
pub struct RequestHash(pub u64);

pub trait CanCache<R: Request>: Cache + Lookup<R::Result, Key = R> + Store<R::Result, Key = RequestHash> {}

impl<R: Request, C: Cache> CanCache<R> for C where C: Lookup<R::Result, Key = R> + Store<R::Result, Key = RequestHash> {}

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
