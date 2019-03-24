use crate::{api::request::Request, error::CacheError, Secondary};
use chrono::NaiveDateTime;
use gdcf_model::{song::NewgroundsSong, user::Creator};
use std::ops::Deref;

pub trait Cache: Clone + Send + Sync + 'static {
    type Err: CacheError;
    type CacheEntryMeta: CacheEntryMeta;

    fn store_secondary(&self, object: &Secondary) -> Result<(), Self::Err>;
}

pub trait Lookup<Obj>: Cache {
    fn lookup(&self, key: u64) -> Result<CacheEntry<Obj, Self>, Self::Err>;
}

pub trait Store<Obj>: Cache {
    fn store(&mut self, obj: &Obj, key: u64) -> Result<Self::CacheEntryMeta, Self::Err>;
}

#[derive(Debug)]
pub struct RequestHash(pub u64);

pub trait CanCache<R: Request>: Cache + Lookup<R::Result> + Store<R::Result> {
    fn lookup_request(&self, request: &R) -> Result<CacheEntry<R::Result, Self>, Self::Err> {
        self.lookup(request.key())
    }
}

impl<R: Request, C: Cache> CanCache<R> for C where C: Lookup<R::Result> + Store<R::Result> {}

#[derive(Debug, PartialEq)]
pub struct CacheEntry<T, C: Cache> {
    pub(crate) object: T,
    pub(crate) metadata: C::CacheEntryMeta,
}

impl<T, C: Cache> CacheEntry<T, C> {
    pub fn meta(&self) -> &C::CacheEntryMeta {
        &self.metadata
    }

    pub fn into_inner(self) -> T {
        self.object
    }

    pub fn is_expired(&self) -> bool {
        self.metadata.is_expired()
    }
}

pub trait CacheEntryMeta {
    fn is_expired(&self) -> bool;
}
