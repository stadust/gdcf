use crate::{
    api::request::Request,
    error::{ApiError, CacheError, GdcfError},
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
    DeducedAbsent,
    MarkedAbsent(C::CacheEntryMeta),
    Cached(T, C::CacheEntryMeta),
}

impl<T, C: Cache> CacheEntry<T, C> {
    pub fn new(object: T, metadata: C::CacheEntryMeta) -> Self {
        CacheEntry::Cached(object, metadata)
    }

    pub fn absent(metadata: C::CacheEntryMeta) -> Self {
        CacheEntry::MarkedAbsent(metadata)
    }

    pub fn is_expired(&self) -> bool {
        match self {
            CacheEntry::DeducedAbsent => false,
            CacheEntry::MarkedAbsent(meta) | CacheEntry::Cached(_, meta) => meta.is_expired(),
        }
    }

    pub fn is_absent(&self) -> bool {
        match self {
            CacheEntry::Cached(..) => false,
            _ => true,
        }
    }

    pub(crate) fn combine<AddOn, R>(
        self, other: CacheEntry<AddOn, C>, combinator: impl Fn(T, Option<AddOn>) -> Option<R>,
    ) -> CacheEntry<R, C> {
        match self {
            CacheEntry::DeducedAbsent => CacheEntry::DeducedAbsent,
            CacheEntry::MarkedAbsent(meta) => CacheEntry::MarkedAbsent(meta),
            CacheEntry::Cached(object, meta) =>
                match other {
                    CacheEntry::MarkedAbsent(_) | CacheEntry::DeducedAbsent =>
                        match combinator(object, None) {
                            Some(combined) => CacheEntry::Cached(combined, meta),
                            None =>
                                match other {
                                    CacheEntry::DeducedAbsent => CacheEntry::DeducedAbsent,
                                    CacheEntry::MarkedAbsent(absent_meta) => CacheEntry::MarkedAbsent(absent_meta),
                                    _ => unreachable!(),
                                },
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
            CacheEntry::DeducedAbsent => Ok((CacheEntry::DeducedAbsent, None)),
            CacheEntry::MarkedAbsent(meta) => Ok((CacheEntry::MarkedAbsent(meta), None)),
            CacheEntry::Cached(ref object, _) => {
                let entry = lookup(object)?;
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
// FIXME: If impl specialization ever gets stabilized, this looks like something that could benefit
// from it.
impl<T, C: Cache> CacheEntry<Vec<T>, C> {
    pub(crate) fn extend_all<A: ApiError, AddOn, U, Look, Req, Comb, Fut>(
        self, lookup: Look, request: Req, combinator: Comb,
    ) -> Result<
        (
            CacheEntry<Vec<U>, C>,
            Option<impl Future<Item = CacheEntry<Vec<U>, C>, Error = GdcfError<A, C::Err>>>,
        ),
        C::Err,
    >
    where
        T: Clone,
        Look: Fn(&T) -> Result<CacheEntry<AddOn, C>, C::Err>,
        Req: Fn(&T) -> Fut,
        Comb: Copy + Fn(T, Option<AddOn>) -> Option<U>,
        Fut: Future<Item = CacheEntry<AddOn, C>, Error = GdcfError<A, C::Err>>,
    {
        match self {
            CacheEntry::DeducedAbsent => Ok((CacheEntry::DeducedAbsent, None)),
            CacheEntry::MarkedAbsent(meta) => Ok((CacheEntry::MarkedAbsent(meta), None)),
            CacheEntry::Cached(objects, meta) => {
                let mut combined = Vec::new();
                let mut futures = Vec::new();

                for object in objects {
                    let addon_entry = lookup(&object)?;

                    if addon_entry.is_expired() {
                        let entry = CacheEntry::Cached(object.clone(), meta);

                        futures.push(request(&object).map(move |addon| entry.combine(addon, combinator)))
                    };

                    if let CacheEntry::Cached(object, _) = CacheEntry::Cached(object, meta).combine(addon_entry, combinator) {
                        combined.push(object)
                    }
                }

                let combined_entry = CacheEntry::Cached(combined, meta);

                if futures.is_empty() {
                    Ok((combined_entry, None))
                } else {
                    let future = futures::future::join_all(futures).map(move |entries| {
                        CacheEntry::Cached(
                            entries
                                .into_iter()
                                .filter_map(|entry| {
                                    match entry {
                                        CacheEntry::Cached(object, _) => Some(object),
                                        _ => None,
                                    }
                                })
                                .collect(),
                            meta,
                        )
                    });

                    Ok((combined_entry, Some(future)))
                }
            },
        }
    }
}

pub trait CacheEntryMeta: Copy + Send + Sync + 'static {
    fn is_expired(&self) -> bool;
    fn is_absent(&self) -> bool;
}
