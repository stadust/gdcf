use crate::{api::request::Request, error::CacheError, Secondary};
use chrono::NaiveDateTime;
use gdcf_model::{song::NewgroundsSong, user::Creator};
use std::ops::Deref;

pub trait Cache: Clone + Send + Sync + 'static {
    type Err: CacheError;
    type CacheEntryMeta;

    fn store_secondary(&self, object: &Secondary) -> Result<(), Self::Err>;
}

pub trait Lookup<Obj>: Cache {
    type Key;

    fn lookup(&self, id: &Self::Key) -> Result<CacheEntry<Obj, Self>, Self::Err>;
    fn is_expired(&self, entry: &CacheEntry<Obj, Self>) -> bool;
}

pub trait Store<Obj>: Cache {
    type Key;

    fn store(&mut self, obj: &Obj, key: Self::Key) -> Result<(), Self::Err>;
}

#[derive(Debug)]
pub struct RequestHash(pub u64);

pub trait CanCache<R: Request>: Cache + Lookup<R::Result, Key = R> + Store<R::Result, Key = RequestHash> {}

impl<R: Request, C: Cache> CanCache<R> for C where C: Lookup<R::Result, Key = R> + Store<R::Result, Key = RequestHash> {}

#[derive(Debug, PartialEq)]
pub struct CacheEntry<T, C: Cache> {
    object: T,
    metadata: C::CacheEntryMeta,
}

impl<T, C: Cache> CacheEntry<T, C> {
    pub fn meta(&self) -> &C::CacheEntryMeta {
        &self.metadata
    }

    pub fn into_inner(self) -> T {
        self.object
    }
}