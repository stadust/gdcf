use crate::{
    api::{client::MakeRequest, ApiClient},
    cache::{Cache, CacheEntry, CanCache, Store},
    error::GdcfError,
    extend::{Extendable, ExtensionModes},
    Gdcf,
};
use futures::{Async, Future};
use gdcf_model::{song::NewgroundsSong, user::Creator};

enum ExtensionFuture<From, A, C, Into, Ext, E>
where
    A: ApiClient + MakeRequest<E::Request>,
    C: Store<Creator> + Store<NewgroundsSong> + CanCache<E::Request>,
    From: Future<Item = CacheEntry<E, C::CacheEntryMeta>, Error = GdcfError<A::Err, C::Err>>,
    A: ApiClient,
    C: Cache,
    E: Extendable<C, Into, Ext>,
{
    WaitingOnInner(Gdcf<A, C>, bool, From),
    Extending(C, C::CacheEntryMeta, ExtensionModes<A, C, Into, Ext, E>),
    Exhausted,
}

enum ExtendManyFuture<From, A, C, Into, Ext, E>
where
    A: ApiClient + MakeRequest<E::Request>,
    C: Store<Creator> + Store<NewgroundsSong> + CanCache<E::Request>,
    From: Future<Item = CacheEntry<Vec<E>, C::CacheEntryMeta>, Error = GdcfError<A::Err, C::Err>>,
    A: ApiClient,
    C: Cache,
    E: Extendable<C, Into, Ext>,
{
    WaitingOnInner(Gdcf<A, C>, bool, From),
    Extending(C, C::CacheEntryMeta, Vec<ExtensionModes<A, C, Into, Ext, E>>),
    Exhausted,
}

impl<From, A, C, Into, Ext, E> Future for ExtendManyFuture<From, A, C, Into, Ext, E>
where
    A: ApiClient + MakeRequest<E::Request>,
    C: Store<Creator> + Store<NewgroundsSong> + CanCache<E::Request>,
    From: Future<Item = CacheEntry<Vec<E>, C::CacheEntryMeta>, Error = GdcfError<A::Err, C::Err>>,
    A: ApiClient,
    C: Cache,
    E: Extendable<C, Into, Ext>,
{
    type Error = GdcfError<A::Err, C::Err>;
    type Item = CacheEntry<Vec<Into>, C::CacheEntryMeta>;

    fn poll(&mut self) -> Result<Async<Self::Item>, Self::Error> {
        let (return_value, next_state) = match self {
            ExtendManyFuture::WaitingOnInner(gdcf, forces_refresh, inner) =>
                match inner.poll()? {
                    Async::Ready(base) =>
                        match base {
                            CacheEntry::DeducedAbsent => (Async::Ready(CacheEntry::DeducedAbsent), ExtendManyFuture::Exhausted),
                            CacheEntry::MarkedAbsent(meta) => (Async::Ready(CacheEntry::MarkedAbsent(meta)), ExtendManyFuture::Exhausted),
                            CacheEntry::Missing => unreachable!(),
                            CacheEntry::Cached(objects, meta) => {
                                // TODO: figure out if this is really needed
                                futures::task::current().notify();

                                (
                                    Async::NotReady,
                                    ExtendManyFuture::Extending(
                                        gdcf.cache(),
                                        meta,
                                        objects
                                            .into_iter()
                                            .map(|object| ExtensionModes::new(object, gdcf, *forces_refresh))
                                            .collect::<Result<Vec<_>, _>>()?,
                                    ),
                                )
                            },
                        },
                    Async::NotReady => return Ok(Async::NotReady),
                },
            ExtendManyFuture::Extending(ref cache, _, ref mut extensions) => {
                let mut are_we_there_yet = true;

                for extension in extensions {
                    match extension {
                        ExtensionModes::ExtensionWasOutdated(_, _, ref mut future)
                        | ExtensionModes::ExtensionWasMissing(_, ref mut future) =>
                            match future.poll()? {
                                Async::NotReady => are_we_there_yet = false,
                                Async::Ready(CacheEntry::Missing) => unreachable!(),
                                Async::Ready(CacheEntry::DeducedAbsent) | Async::Ready(CacheEntry::MarkedAbsent(_)) =>
                                    if let ExtensionModes::ExtensionWasMissing(to_extend, _)
                                    | ExtensionModes::ExtensionWasOutdated(to_extend, ..) =
                                        std::mem::replace(extension, ExtensionModes::FixMeItIsLateAndICannotThinkOfABetterSolution)
                                    {
                                        std::mem::replace(
                                            extension,
                                            ExtensionModes::ExtensionWasCached(
                                                to_extend
                                                    .combine(E::on_extension_absent().ok_or(GdcfError::ConsistencyAssumptionViolated)?),
                                            ),
                                        );
                                    },
                                Async::Ready(CacheEntry::Cached(request_result, _)) => {
                                    if let ExtensionModes::ExtensionWasMissing(to_extend, _)
                                    | ExtensionModes::ExtensionWasOutdated(to_extend, ..) =
                                        std::mem::replace(extension, ExtensionModes::FixMeItIsLateAndICannotThinkOfABetterSolution)
                                    {
                                        let ext = to_extend.lookup_extension(cache, request_result).map_err(GdcfError::Cache)?;

                                        std::mem::replace(extension, ExtensionModes::ExtensionWasCached(to_extend.combine(ext)));
                                    }
                                },
                            },
                        _ => (),
                    }
                }

                if are_we_there_yet {
                    if let ExtendManyFuture::Extending(_, meta, extensions) = std::mem::replace(self, ExtendManyFuture::Exhausted) {
                        return Ok(Async::Ready(CacheEntry::Cached(
                            extensions
                                .into_iter()
                                .map(|extension| {
                                    if let ExtensionModes::ExtensionWasCached(object) = extension {
                                        object
                                    } else {
                                        unreachable!()
                                    }
                                })
                                .collect(),
                            meta,
                        )))
                    } else {
                        unreachable!()
                    }
                } else {
                    return Ok(Async::NotReady)
                }
            },
            _ => unimplemented!(),
            ExtendManyFuture::Exhausted => panic!("Future already polled to completion"),
        };

        std::mem::replace(self, next_state);

        Ok(return_value)
    }
}

impl<From, A, C, Into, Ext, E> Future for ExtensionFuture<From, A, C, Into, Ext, E>
where
    A: ApiClient + MakeRequest<E::Request>,
    C: Store<Creator> + Store<NewgroundsSong> + CanCache<E::Request>,
    From: Future<Item = CacheEntry<E, C::CacheEntryMeta>, Error = GdcfError<A::Err, C::Err>>,
    C: Cache,
    E: Extendable<C, Into, Ext>,
{
    type Error = GdcfError<A::Err, C::Err>;
    type Item = CacheEntry<Into, C::CacheEntryMeta>;

    fn poll(&mut self) -> Result<Async<Self::Item>, Self::Error> {
        let (return_value, next_state) = match self {
            ExtensionFuture::WaitingOnInner(gdcf, forces_refresh, inner) =>
                match inner.poll()? {
                    Async::Ready(base) =>
                        match base {
                            CacheEntry::DeducedAbsent => (Async::Ready(CacheEntry::DeducedAbsent), ExtensionFuture::Exhausted),
                            CacheEntry::MarkedAbsent(meta) => (Async::Ready(CacheEntry::MarkedAbsent(meta)), ExtensionFuture::Exhausted),
                            CacheEntry::Missing => unreachable!(),
                            CacheEntry::Cached(object, meta) => {
                                // TODO: figure out if this is really needed
                                futures::task::current().notify();

                                (
                                    Async::NotReady,
                                    ExtensionFuture::Extending(gdcf.cache(), meta, ExtensionModes::new(object, gdcf, *forces_refresh)?),
                                )
                            },
                        },
                    Async::NotReady => return Ok(Async::NotReady),
                },
            ExtensionFuture::Extending(_, _, ExtensionModes::ExtensionWasCached(_)) => {
                if let ExtensionFuture::Extending(_, meta, ExtensionModes::ExtensionWasCached(object)) =
                    std::mem::replace(self, ExtensionFuture::Exhausted)
                {
                    return Ok(Async::Ready(CacheEntry::Cached(object, meta)))
                } else {
                    unreachable!()
                }
            },
            ExtensionFuture::Extending(_, _, ExtensionModes::ExtensionWasMissing(_, ref mut refresh_future))
            | ExtensionFuture::Extending(_, _, ExtensionModes::ExtensionWasOutdated(_, _, ref mut refresh_future)) =>
                match refresh_future.poll()? {
                    Async::NotReady => return Ok(Async::NotReady),
                    Async::Ready(CacheEntry::Missing) => unreachable!(),
                    Async::Ready(cache_entry) => {
                        let (cache, base, meta) = match std::mem::replace(self, ExtensionFuture::Exhausted) {
                            ExtensionFuture::Extending(cache, meta, ExtensionModes::ExtensionWasMissing(base, _))
                            | ExtensionFuture::Extending(cache, meta, ExtensionModes::ExtensionWasOutdated(base, ..)) =>
                                (cache, base, meta),
                            _ => unreachable!(),
                        };

                        match cache_entry {
                            CacheEntry::DeducedAbsent | CacheEntry::MarkedAbsent(_) =>
                                return Ok(Async::Ready(CacheEntry::Cached(
                                    base.combine(E::on_extension_absent().ok_or(GdcfError::ConsistencyAssumptionViolated)?),
                                    meta,
                                ))),
                            CacheEntry::Cached(request_result, _) => {
                                let extension = base.lookup_extension(&cache, request_result).map_err(GdcfError::Cache)?;

                                return Ok(Async::Ready(CacheEntry::Cached(base.combine(extension), meta)))
                            },
                            _ => unreachable!(),
                        }
                    },
                },
            _ => unreachable!(),
            ExtensionFuture::Exhausted => panic!("Future already polled to completion"),
        };

        std::mem::replace(self, next_state);

        Ok(return_value)
    }
}
