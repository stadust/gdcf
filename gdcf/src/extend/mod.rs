use crate::{
    api::{client::MakeRequest, request::Request, ApiClient},
    cache::{Cache, CacheEntry, CanCache, Store},
    error::GdcfError,
    future::refresh::RefreshCacheFuture,
    EitherOrBoth, Gdcf,
};
use gdcf_model::{song::NewgroundsSong, user::Creator};

pub mod level;
pub mod user;

pub trait Extendable<C: Cache, Into, Ext> {
    type Request: Request;

    fn lookup_extension(&self, cache: &C, request_result: <Self::Request as Request>::Result) -> Result<Ext, C::Err>;

    fn on_extension_absent() -> Option<Ext>;

    fn extension_request(&self) -> Self::Request;

    // TODO: maybe put this into a Combinable trait
    fn combine(self, addon: Ext) -> Into;
}

#[allow(missing_debug_implementations)]
pub enum ExtensionModes<A, C, Into, Ext, E>
where
    A: ApiClient + MakeRequest<E::Request>,
    C: Store<Creator> + Store<NewgroundsSong> + CanCache<E::Request>,
    A: ApiClient,
    C: Cache,
    E: Extendable<C, Into, Ext>,
{
    ExtensionWasCached(Into),
    ExtensionWasOutdated(E, Ext, RefreshCacheFuture<E::Request, A, C>),
    ExtensionWasMissing(E, RefreshCacheFuture<E::Request, A, C>),
    FixMeItIsLateAndICannotThinkOfABetterSolution,
}

impl<A, C, Into, Ext, E> ExtensionModes<A, C, Into, Ext, E>
where
    A: ApiClient + MakeRequest<E::Request>,
    C: Store<Creator> + Store<NewgroundsSong> + CanCache<E::Request>,
    C: Cache,
    E: Extendable<C, Into, Ext>,
{
    pub(crate) fn new(to_extend: E, gdcf: &Gdcf<A, C>, force_refresh: bool) -> Result<Self, GdcfError<A::Err, C::Err>> {
        let cache = gdcf.cache();

        // FIXME: add set_force_refresh(bool) to Request trait
        //let request = if force_refresh { to_extend.extension_request().force_refresh()} else
        // {to_extend.extension_request()};
        let request = to_extend.extension_request();

        let mode = match gdcf.process(request).map_err(GdcfError::Cache)? {
            // Not possible, we'd have gotten EitherOrBoth::B because of how `process` works
            EitherOrBoth::Both(CacheEntry::Missing, _) | EitherOrBoth::A(CacheEntry::Missing) => unreachable!(),
            // Up-to-date absent marker for extension request result. However, we cannot rely on this for this!
            // This violates snapshot consistency! TOdO: document
            EitherOrBoth::A(CacheEntry::DeducedAbsent) | EitherOrBoth::A(CacheEntry::MarkedAbsent(_)) =>
                match E::on_extension_absent() {
                    Some(default_extension) => ExtensionModes::ExtensionWasCached(to_extend.combine(default_extension)),
                    None => {
                        let request = to_extend.extension_request();

                        ExtensionModes::ExtensionWasMissing(to_extend, gdcf.refresh(request))
                    },
                },
            // Up-to-date extension request result
            EitherOrBoth::A(CacheEntry::Cached(request_result, _)) => {
                let extension = to_extend.lookup_extension(&cache, request_result).map_err(GdcfError::Cache)?;
                ExtensionModes::ExtensionWasCached(to_extend.combine(extension))
            },
            // Missing extension request result cache entry
            EitherOrBoth::B(refresh_future) => ExtensionModes::ExtensionWasMissing(to_extend, refresh_future),
            // Outdated absent marker
            EitherOrBoth::Both(CacheEntry::MarkedAbsent(_), refresh_future)
            | EitherOrBoth::Both(CacheEntry::DeducedAbsent, refresh_future) =>
                match E::on_extension_absent() {
                    Some(default_extension) => ExtensionModes::ExtensionWasOutdated(to_extend, default_extension, refresh_future),
                    None => {
                        let request = to_extend.extension_request();

                        ExtensionModes::ExtensionWasMissing(to_extend, gdcf.refresh(request))
                    },
                },
            // Outdated entry
            EitherOrBoth::Both(CacheEntry::Cached(extension, _), refresh_future) => {
                let extension = to_extend.lookup_extension(&cache, extension).map_err(GdcfError::Cache)?;

                ExtensionModes::ExtensionWasOutdated(to_extend, extension, refresh_future)
            },
        };

        Ok(mode)
    }
}
