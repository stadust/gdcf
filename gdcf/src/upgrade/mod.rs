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

pub trait Upgrade<C: Cache, Into> {
    type Request: Request;
    type From;
    type Upgrade;

    /// Gets the request that needs to be made to retrieve the data for this upgrade
    ///
    /// Returning [`None`] indicates that an upgrade of this object is not possible and will cause a
    /// call to [`Upgrade::default_upgrade`]
    fn upgrade_request(from: &Self::From) -> Option<Self::Request>;

    fn current(&self) -> &Self::From;

    /// Gets the default [`Upgrade::Upgrade`] object to be used if an upgrade wasn't possible (see
    /// above) or if the request didn't return the required data.
    ///
    /// Returning [`None`] here indicates that no default option is available. That generally means
    /// that the upgrade process has failed completely
    fn default_upgrade() -> Option<Self::Upgrade>;

    fn lookup_upgrade(from: &Self::From, cache: &C, request_result: <Self::Request as Request>::Result) -> Result<Self::Upgrade, C::Err>;

    fn upgrade(self, upgrade: Self::Upgrade) -> Into;
}

#[allow(missing_debug_implementations)]
pub enum UpgradeMode<A, C, Into, E>
where
    A: ApiClient + MakeRequest<E::Request>,
    C: Store<Creator> + Store<NewgroundsSong> + CanCache<E::Request>,
    C: Cache,
    E: Upgrade<C, Into>,
{
    UpgradeCached(Into),
    UpgradeOutdated(E, E::Upgrade, RefreshCacheFuture<E::Request, A, C>),
    UpgradeMissing(E, RefreshCacheFuture<E::Request, A, C>),
    FixMeItIsLateAndICannotThinkOfABetterSolution,
}

impl<A, C, Into, E> UpgradeMode<A, C, Into, E>
where
    A: ApiClient + MakeRequest<E::Request>,
    C: Store<Creator> + Store<NewgroundsSong> + CanCache<E::Request>,
    C: Cache,
    E: Upgrade<C, Into>,
{
    pub(crate) fn new(to_upgrade: E, gdcf: &Gdcf<A, C>, force_refresh: bool) -> Result<Self, GdcfError<A::Err, C::Err>> {
        let cache = gdcf.cache();

        // FIXME: add set_force_refresh(bool) to Request trait
        //let request = if force_refresh { to_extend.extension_request().force_refresh()} else
        // {to_extend.extension_request()};
        let request = match E::upgrade_request(to_upgrade.current()) {
            Some(request) => request,
            None =>
                return Ok(UpgradeMode::UpgradeCached(
                    to_upgrade.upgrade(E::default_upgrade().ok_or(GdcfError::ConsistencyAssumptionViolated)?),
                )),
        };

        let mode = match gdcf.process(request).map_err(GdcfError::Cache)? {
            // Not possible, we'd have gotten EitherOrBoth::B because of how `process` works
            EitherOrBoth::Both(CacheEntry::Missing, _) | EitherOrBoth::A(CacheEntry::Missing) => unreachable!(),

            // Up-to-date absent marker for extension request result. However, we cannot rely on this for this!
            // This violates snapshot consistency! TOdO: document
            EitherOrBoth::A(CacheEntry::DeducedAbsent) | EitherOrBoth::A(CacheEntry::MarkedAbsent(_)) =>
                match E::default_upgrade() {
                    Some(default_extension) => UpgradeMode::UpgradeCached(to_upgrade.upgrade(default_extension)),
                    None =>
                        match E::upgrade_request(to_upgrade.current()) {
                            None =>
                                UpgradeMode::UpgradeCached(
                                    to_upgrade.upgrade(E::default_upgrade().ok_or(GdcfError::ConsistencyAssumptionViolated)?),
                                ),
                            Some(request) => UpgradeMode::UpgradeMissing(to_upgrade, gdcf.refresh(request)),
                        },
                },

            // Up-to-date extension request result
            EitherOrBoth::A(CacheEntry::Cached(request_result, _)) => {
                let upgrade = E::lookup_upgrade(to_upgrade.current(), &cache, request_result).map_err(GdcfError::Cache)?;
                UpgradeMode::UpgradeCached(to_upgrade.upgrade(upgrade))
            },

            // Missing extension request result cache entry
            EitherOrBoth::B(refresh_future) => UpgradeMode::UpgradeMissing(to_upgrade, refresh_future),

            // Outdated absent marker
            EitherOrBoth::Both(CacheEntry::MarkedAbsent(_), refresh_future)
            | EitherOrBoth::Both(CacheEntry::DeducedAbsent, refresh_future) =>
                match E::default_upgrade() {
                    Some(default_extension) => UpgradeMode::UpgradeOutdated(to_upgrade, default_extension, refresh_future),
                    None =>
                        match E::upgrade_request(to_upgrade.current()) {
                            None =>
                                UpgradeMode::UpgradeCached(
                                    to_upgrade.upgrade(E::default_upgrade().ok_or(GdcfError::ConsistencyAssumptionViolated)?),
                                ),
                            Some(request) => UpgradeMode::UpgradeMissing(to_upgrade, gdcf.refresh(request)),
                        },
                },

            // Outdated entry
            EitherOrBoth::Both(CacheEntry::Cached(request_result, _), refresh_future) => {
                let upgrade = E::lookup_upgrade(to_upgrade.current(), &cache, request_result).map_err(GdcfError::Cache)?;

                UpgradeMode::UpgradeOutdated(to_upgrade, upgrade, refresh_future)
            },
        };

        Ok(mode)
    }
}
