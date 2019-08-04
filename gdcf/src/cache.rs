use crate::{
    api::request::Request,
    error::{ApiError, CacheError, GdcfError},
};
use futures::Future;
use std::fmt::{Display, Formatter};

pub trait Cache: Clone + Send + Sync + 'static {
    type CacheEntryMeta: CacheEntryMeta;
    type Err: CacheError;
}

pub trait Lookup<Obj>: Cache {
    fn lookup(&self, key: u64) -> Result<CacheEntry<Obj, Self::CacheEntryMeta>, Self::Err>;
}

pub trait Store<Obj>: Cache {
    fn store(&mut self, obj: &Obj, key: u64) -> Result<Self::CacheEntryMeta, Self::Err>;
    fn mark_absent(&mut self, key: u64) -> Result<Self::CacheEntryMeta, Self::Err>;
}

// TODO: make this private
pub trait CanCache<R: Request>: Cache + Lookup<R::Result> + Store<R::Result> {
    fn lookup_request(&self, request: &R) -> Result<CacheEntry<R::Result, Self::CacheEntryMeta>, Self::Err> {
        self.lookup(request.key())
    }
}

impl<R: Request, C: Cache> CanCache<R> for C where C: Lookup<R::Result> + Store<R::Result> {}

#[derive(Debug, PartialEq, Clone, Copy)]
pub enum CacheEntry<T, Meta: CacheEntryMeta> {
    /// Variant to return if there was no entry at all in the cache regarding a specific request,
    /// meaning the request hasn't been done yet ever
    Missing,

    /// Variant indicating that the there was no entry at all in the cache regarding a specific
    /// request, but it could be deduced from the context that a request that should have caused an
    /// entry has already been made.
    DeducedAbsent,

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
            CacheEntry::DeducedAbsent => write!(f, "Cache entry deduced missing due to server sided data inconsistency"),
            CacheEntry::MarkedAbsent(meta) => write!(f, "{} marked as missing due to empty server response", meta),
            CacheEntry::Cached(object, meta) => write!(f, "Cached {}, {}", object, meta),
        }
    }
}

impl<T, Meta: CacheEntryMeta> CacheEntry<T, Meta> {
    pub fn new(object: T, metadata: Meta) -> Self {
        CacheEntry::Cached(object, metadata)
    }

    pub fn absent(metadata: Meta) -> Self {
        CacheEntry::MarkedAbsent(metadata)
    }

    pub fn is_expired(&self) -> bool {
        match self {
            CacheEntry::Missing => true,
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
        self,
        other: CacheEntry<AddOn, Meta>,
        combinator: impl Fn(T, Option<AddOn>) -> Option<R>,
    ) -> CacheEntry<R, Meta> {
        match self {
            CacheEntry::Missing => CacheEntry::Missing,
            CacheEntry::DeducedAbsent => CacheEntry::DeducedAbsent,
            CacheEntry::MarkedAbsent(meta) => CacheEntry::MarkedAbsent(meta),
            CacheEntry::Cached(object, meta) =>
                match other {
                    CacheEntry::Missing => CacheEntry::Missing,
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

    pub(crate) fn extend<A: ApiError, C: CacheError, AddOn, U, Look, Req, Comb, Fut>(
        self,
        lookup: Look,
        request: Req,
        combinator: Comb,
    ) -> Result<
        (
            CacheEntry<U, Meta>,
            Option<impl Future<Item = CacheEntry<U, Meta>, Error = GdcfError<A, C>>>,
        ),
        C,
    >
    where
        T: Clone,
        Look: FnOnce(&T) -> Result<CacheEntry<AddOn, Meta>, C>,
        Req: FnOnce(&T) -> Fut,
        Comb: Copy + Fn(T, Option<AddOn>) -> Option<U>,
        Fut: Future<Item = CacheEntry<AddOn, Meta>, Error = GdcfError<A, C>>,
    {
        match self {
            CacheEntry::DeducedAbsent => Ok((CacheEntry::DeducedAbsent, None)),
            CacheEntry::MarkedAbsent(meta) => Ok((CacheEntry::MarkedAbsent(meta), None)),
            CacheEntry::Cached(ref object, _) => {
                // What happens when this one is CacheEntry::Missing:
                // + entry.is_expired() will return true
                // + combine will return CacheEntry::Missing
                // --> We get the correct behavior
                let entry = lookup(object)?;
                let future = if entry.is_expired() {
                    let clone = self.clone();

                    Some(request(object).map(move |addon| clone.combine(addon, combinator)))
                } else {
                    None
                };

                Ok((self.combine(entry, combinator), future))
            },
            CacheEntry::Missing => Ok((CacheEntry::Missing, None)),
        }
    }
}
// FIXME: If impl specialization ever gets stabilized, this looks like something that could benefit
// from it.
impl<T, Meta: CacheEntryMeta> CacheEntry<Vec<T>, Meta> {
    pub(crate) fn extend_all<A: ApiError, C: CacheError, AddOn, U, Look, Req, Comb, Fut>(
        self,
        lookup: Look,
        request: Req,
        combinator: Comb,
    ) -> Result<
        (
            CacheEntry<Vec<U>, Meta>,
            Option<impl Future<Item = CacheEntry<Vec<U>, Meta>, Error = GdcfError<A, C>>>,
        ),
        C,
    >
    where
        T: Clone,
        Look: Fn(&T) -> Result<CacheEntry<AddOn, Meta>, C>,
        Req: Fn(&T) -> Fut,
        Comb: Copy + Fn(T, Option<AddOn>) -> Option<U>,
        Fut: Future<Item = CacheEntry<AddOn, Meta>, Error = GdcfError<A, C>>,
    {
        match self {
            CacheEntry::Missing => Ok((CacheEntry::Missing, None)),
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

pub trait CacheUserExt {
    /// Tries to somehow map the username to an account ID, using whatever data is available
    ///
    /// If the user associated with this name is found in either the [`SearchedUser`] or [`User`]
    /// cache, this function must never, under any circumstances, return [`None`]. Doing so is
    /// considered a bug which the library may or may not recover from.
    fn username_to_account_id(&self, name: &str) -> Option<u64>;
}
