//! Module containing cache related traits/structs

use crate::{api::request::Request, error::CacheError};
use derive_more::Display;
use gdcf_model::{song::NewgroundsSong, user::Creator};
use std::fmt::{Display, Formatter};

pub trait Cache: Clone + Send + Sync + 'static {
    type CacheEntryMeta: CacheEntryMeta;
    type Err: CacheError;
}

#[derive(Debug, Display)]
pub struct NewgroundsSongKey(pub u64);

#[derive(Debug, Display)]
pub struct CreatorKey(pub u64);

pub trait Key {
    type Result;
}

impl<R: Request> Key for R {
    type Result = <R as Request>::Result;
}

impl Key for NewgroundsSongKey {
    type Result = NewgroundsSong;
}

impl Key for CreatorKey {
    type Result = Creator;
}

pub trait Lookup<K: Key>: Cache {
    // TODO: maybe an exists method?
    fn lookup(&self, key: &K) -> Result<CacheEntry<K::Result, Self::CacheEntryMeta>, Self::Err>;
}

pub trait Store<K: Key>: Cache {
    fn store(&mut self, obj: &K::Result, key: &K) -> Result<Self::CacheEntryMeta, Self::Err>;
    fn mark_absent(&mut self, key: &K) -> Result<Self::CacheEntryMeta, Self::Err>;
}

// FIXME: One they are stabilized, use a trait alias here
pub trait CanCache<K: Key>: Lookup<K> + Store<K> {}

impl<K: Key, C: Cache> CanCache<K> for C where C: Store<K> + Lookup<K> {}

/// Struct modelling the result of some GDCF request
#[derive(Debug, PartialEq, Clone)]
pub enum CacheEntry<T, Meta: CacheEntryMeta> {
    /// Variant to return if there was no entry at all in the cache regarding a specific request,
    /// meaning the request hasn't been done yet ever
    Missing,

    /// Variant indicating that a request was already made previously, but the server indicated
    /// returned an empty response
    MarkedAbsent(Meta),

    /// Variant indicating that a request was already made, and its results were stored.
    Cached(T, Meta),
}

impl<T: Display, Meta: CacheEntryMeta + Display> Display for CacheEntry<T, Meta> {
    fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
        match self {
            CacheEntry::Missing => write!(f, "Cache entry missing"),
            CacheEntry::MarkedAbsent(meta) => write!(f, "{} marked as missing due to empty server response", meta),
            CacheEntry::Cached(object, meta) => write!(f, "Cached {}, {}", object, meta),
        }
    }
}

impl<T, Meta: CacheEntryMeta> CacheEntry<T, Meta> {
    pub fn is_expired(&self) -> bool {
        match self {
            CacheEntry::Missing => true,
            CacheEntry::MarkedAbsent(meta) | CacheEntry::Cached(_, meta) => meta.is_expired(),
        }
    }

    pub fn is_absent(&self) -> bool {
        match self {
            CacheEntry::Cached(..) => false,
            _ => true,
        }
    }

    pub fn map<U>(self, f: impl FnOnce(T) -> U) -> CacheEntry<U, Meta> {
        match self {
            CacheEntry::Missing => CacheEntry::Missing,
            CacheEntry::MarkedAbsent(absent_meta) => CacheEntry::MarkedAbsent(absent_meta),
            CacheEntry::Cached(object, meta) => CacheEntry::Cached(f(object), meta),
        }
    }

    pub(crate) fn map_empty<U>(self) -> CacheEntry<U, Meta> {
        self.map(|_| panic!("CacheEntry::map_empty called on `Cached` variant"))
    }
}

pub trait CacheEntryMeta: Clone + std::fmt::Debug + Copy + Send + Sync + 'static {
    fn is_expired(&self) -> bool;
    fn is_absent(&self) -> bool;
}
