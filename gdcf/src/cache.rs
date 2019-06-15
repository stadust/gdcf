use crate::{
    api::request::Request,
    error::{ApiError, CacheError, GdcfError},
    Secondary,
};
use futures::Future;

pub trait Cache: Clone + Send + Sync + 'static {
    type CacheEntryMeta: CacheEntryMeta;
    type Err: CacheError;
}

pub trait Lookup<Obj>: Cache {
    fn lookup(&self, key: u64) -> Result<CacheEntry<Obj, Self>, Self::Err>;
}

pub trait Store<Obj>: Cache {
    fn store(&mut self, obj: &Obj, key: u64) -> Result<Self::CacheEntryMeta, Self::Err>;
    fn mark_absent(&mut self, key: u64) -> Result<Self::CacheEntryMeta, Self::Err>;
}

// TODO: make this private
pub trait CanCache<R: Request>: Cache + Lookup<R::Result> + Store<R::Result> {
    fn lookup_request(&self, request: &R) -> Result<CacheEntry<R::Result, Self>, Self::Err> {
        self.lookup(request.key())
    }
}

impl<R: Request, C: Cache> CanCache<R> for C where C: Lookup<R::Result> + Store<R::Result> {}

#[derive(Debug, PartialEq, Clone, Copy)]
pub enum CacheEntry<T, C: Cache> {
    Absent(C::CacheEntryMeta),
    Cached(T, C::CacheEntryMeta),
}
/*
pub struct CacheEntry<T, C: Cache> {
    pub object: Option<T>,
    pub metadata: C::CacheEntryMeta,
}*/

impl<T, C: Cache> CacheEntry<T, C> {
    pub fn new(object: T, metadata: C::CacheEntryMeta) -> Self {
        CacheEntry::Cached(object, metadata)
    }

    pub fn absent(metadata: C::CacheEntryMeta) -> Self {
        CacheEntry::Absent(metadata)
    }

    pub fn meta(&self) -> &C::CacheEntryMeta {
        match self {
            CacheEntry::Absent(meta) | CacheEntry::Cached(_, meta) => meta,
        }
    }

    pub fn is_expired(&self) -> bool {
        self.meta().is_expired()
    }

    pub fn is_absent(&self) -> bool {
        match self {
            CacheEntry::Absent(_) => true,
            _ => false,
        }
    }

    pub(crate) fn combine<AddOn, R>(
        self, other: CacheEntry<AddOn, C>, combinator: impl Fn(T, Option<AddOn>) -> Option<R>,
    ) -> CacheEntry<R, C> {
        match self {
            CacheEntry::Absent(meta) => CacheEntry::Absent(meta),
            CacheEntry::Cached(object, meta) =>
                match other {
                    CacheEntry::Absent(absent_meta) =>
                        match combinator(object, None) {
                            Some(combined) => CacheEntry::Cached(combined, meta),
                            None => CacheEntry::Absent(absent_meta),
                        },
                    CacheEntry::Cached(addon, _) => CacheEntry::Cached(combinator(object, Some(addon)).unwrap(), meta),
                },
        }
    }

    pub(crate) fn extend<A: ApiError, AddOn, U, Look, Req, Comb, Fut>(
        self, lookup: Look, request: Req, combinator: Comb,
    ) -> Result<
        (
            CacheEntry<U, C>,
            Option<impl Future<Item = CacheEntry<U, C>, Error = GdcfError<A, C::Err>>>,
        ),
        C::Err,
    >
    where
        T: Clone,
        Look: FnOnce(&T) -> Result<CacheEntry<AddOn, C>, C::Err>,
        Req: FnOnce(&T) -> Fut,
        Comb: Copy + Fn(T, Option<AddOn>) -> Option<U>,
        Fut: Future<Item = CacheEntry<AddOn, C>, Error = GdcfError<A, C::Err>>,
    {
        match self {
            CacheEntry::Absent(meta) => Ok((CacheEntry::Absent(meta), None)),
            CacheEntry::Cached(ref object, ref meta) => {
                let entry = lookup(&object)?;
                let future = if entry.is_expired() {
                    let clone = self.clone();

                    Some(request(object).map(move |addon| clone.combine(addon, combinator)))
                } else {
                    None
                };

                Ok((self.combine(entry, combinator), future))
            },
        }
    }
}

pub trait CacheEntryMeta: Clone + Send + Sync + 'static {
    fn is_expired(&self) -> bool;
    fn is_absent(&self) -> bool;
}
