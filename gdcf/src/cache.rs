use crate::{api::request::Request, error::CacheError, Secondary};

pub trait Cache: Clone + Send + Sync + 'static {
    type CacheEntryMeta: CacheEntryMeta;
    type Err: CacheError;
}

pub trait Lookup<Obj>: Cache {
    fn lookup(&self, key: u64) -> Result<CacheEntry<Obj, Self>, Self::Err>;
}

pub trait Store<Obj>: Cache {
    fn store(&mut self, obj: &Obj, key: u64) -> Result<Self::CacheEntryMeta, Self::Err>;
}

pub trait CanCache<R: Request>: Cache + Lookup<R::Result> + Store<R::Result> {
    fn lookup_request(&self, request: &R) -> Result<CacheEntry<R::Result, Self>, Self::Err> {
        self.lookup(request.key())
    }
}

impl<R: Request, C: Cache> CanCache<R> for C where C: Lookup<R::Result> + Store<R::Result> {}

#[derive(Debug, PartialEq, Clone, Copy)]
pub struct CacheEntry<T, C: Cache> {
    pub object: T,
    pub metadata: C::CacheEntryMeta,
}

impl<T, C: Cache> CacheEntry<T, C> {
    pub fn new(object: T, metadata: C::CacheEntryMeta) -> Self {
        Self { object, metadata }
    }

    pub fn meta(&self) -> &C::CacheEntryMeta {
        &self.metadata
    }

    pub fn into_inner(self) -> T {
        self.object
    }

    pub fn inner(&self) -> &T {
        &self.object
    }

    pub fn is_expired(&self) -> bool {
        self.metadata.is_expired()
    }
}

pub trait CacheEntryMeta: Clone + Send + Sync + 'static {
    fn is_expired(&self) -> bool;
}
