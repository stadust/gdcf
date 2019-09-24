use crate::{
    api::{client::MakeRequest, request::Request, ApiClient},
    cache::{Cache, CacheEntry, CanCache, Store},
    error::Error,
    future::{process::ProcessRequestFutureState, refresh::RefreshCacheFuture},
    Gdcf,
};
use gdcf_model::{song::NewgroundsSong, user::Creator};

pub mod level;
pub mod user;

pub trait Upgradable<C: Cache, Into>: Sized {
    type From;
    type Request: Request;
    type Upgrade;

    /// Gets the request that needs to be made to retrieve the data for this upgrade
    ///
    /// Returning [`None`] indicates that an upgrade of this object is not possible and will cause a
    /// call to [`Upgrade::default_upgrade`]
    fn upgrade_request(&self) -> Option<Self::Request>;

    /// Gets the default [`Upgrade::Upgrade`] object to be used if an upgrade wasn't possible (see
    /// above) or if the request didn't return the required data.
    ///
    /// Returning [`None`] here indicates that no default option is available. That generally means
    /// that the upgrade process has failed completely
    fn default_upgrade() -> Option<Self::Upgrade>;

    fn lookup_upgrade(&self, cache: &C, request_result: <Self::Request as Request>::Result) -> Result<Self::Upgrade, C::Err>;

    fn upgrade(self, upgrade: Self::Upgrade) -> (Into, Self::From);
    fn downgrade(upgraded: Into, downgrade: Self::From) -> (Self, Self::Upgrade);
}

pub(crate) enum PendingUpgrade<A, C, Into, U>
where
    A: ApiClient + MakeRequest<U::Request>,
    C: Cache + Store<Creator> + Store<NewgroundsSong> + CanCache<U::Request>,
    U: Upgradable<C, Into>,
{
    Cached(Into),
    Outdated(U, U::Upgrade, RefreshCacheFuture<U::Request, A, C>),
    Missing(U, RefreshCacheFuture<U::Request, A, C>),
}

impl<A, C, Into, U> std::fmt::Debug for PendingUpgrade<A, C, Into, U>
where
    A: ApiClient + MakeRequest<U::Request>,
    C: Cache + Store<Creator> + Store<NewgroundsSong> + CanCache<U::Request>,
    U: Upgradable<C, Into> + std::fmt::Debug,
    U::Upgrade: std::fmt::Debug,
    Into: std::fmt::Debug,
{
    fn fmt(&self, fmt: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            PendingUpgrade::Cached(cached) => fmt.debug_tuple("UpgradeCached").field(cached).finish(),
            PendingUpgrade::Outdated(to_extend, cached_extension, future) =>
                fmt.debug_tuple("UpgradeOutdated")
                    .field(to_extend)
                    .field(cached_extension)
                    .field(future)
                    .finish(),
            PendingUpgrade::Missing(to_extend, future) => fmt.debug_tuple("UpgradeMissing").field(to_extend).field(future).finish(),
        }
    }
}

impl<A, C, Into, U> PendingUpgrade<A, C, Into, U>
where
    A: ApiClient + MakeRequest<U::Request>,
    C: Cache + Store<Creator> + Store<NewgroundsSong> + CanCache<U::Request>,
    U: Upgradable<C, Into>,
{
    pub(crate) fn cached(to_upgrade: U, upgrade: U::Upgrade) -> Self {
        PendingUpgrade::Cached(to_upgrade.upgrade(upgrade).0)
    }

    pub(crate) fn default_upgrade(to_upgrade: U) -> Result<Self, Error<A::Err, C::Err>> {
        Ok(PendingUpgrade::Cached(
            to_upgrade.upgrade(U::default_upgrade().ok_or(Error::UnexpectedlyAbsent)?).0,
        ))
    }

    pub(crate) fn future(&mut self) -> Option<&mut RefreshCacheFuture<U::Request, A, C>> {
        match self {
            PendingUpgrade::Outdated(_, _, ref mut future) | PendingUpgrade::Missing(_, ref mut future) => Some(future),
            _ => None,
        }
    }

    pub(crate) fn into_upgradable(self) -> Option<U> {
        match self {
            PendingUpgrade::Outdated(to_upgrade, ..) | PendingUpgrade::Missing(to_upgrade, _) => Some(to_upgrade),
            _ => None,
        }
    }

    pub(crate) fn new(to_upgrade: U, gdcf: &Gdcf<A, C>, force_refresh: bool) -> Result<Self, Error<A::Err, C::Err>> {
        let cache = gdcf.cache();

        let mut request = match U::upgrade_request(&to_upgrade) {
            Some(request) => request,
            None => return Self::default_upgrade(to_upgrade),
        };

        if force_refresh {
            request.set_force_refresh(true);
        }

        let mode = match gdcf.process(&request).map_err(Error::Cache)? {
            // impossible variants
            ProcessRequestFutureState::Outdated(CacheEntry::Missing, _) | ProcessRequestFutureState::UpToDate(CacheEntry::Missing) =>
                unreachable!(),

            // Up-to-date absent marker for extension request result. However, we cannot rely on this for this!
            // This violates snapshot consistency! TODO: document
            ProcessRequestFutureState::UpToDate(CacheEntry::DeducedAbsent)
            | ProcessRequestFutureState::UpToDate(CacheEntry::MarkedAbsent(_)) =>
            // TODO: investigate what the fuck I have done here
                match U::default_upgrade() {
                    Some(default_upgrade) => Self::cached(to_upgrade, default_upgrade),
                    None =>
                        match U::upgrade_request(&to_upgrade) {
                            None => Self::default_upgrade(to_upgrade)?,
                            Some(request) => PendingUpgrade::Missing(to_upgrade, gdcf.refresh(&request)),
                        },
                },

            ProcessRequestFutureState::UpToDate(CacheEntry::Cached(request_result, _)) => {
                // Up-to-date extension request result
                let upgrade = U::lookup_upgrade(&to_upgrade, &cache, request_result).map_err(Error::Cache)?;
                PendingUpgrade::cached(to_upgrade, upgrade)
            },

            // Missing extension request result cache entry
            ProcessRequestFutureState::Uncached(refresh_future) => PendingUpgrade::Missing(to_upgrade, refresh_future),

            // Outdated absent marker
            ProcessRequestFutureState::Outdated(CacheEntry::MarkedAbsent(_), refresh_future)
            | ProcessRequestFutureState::Outdated(CacheEntry::DeducedAbsent, refresh_future) =>
                match U::default_upgrade() {
                    Some(default_extension) => PendingUpgrade::Outdated(to_upgrade, default_extension, refresh_future),
                    None =>
                        match U::upgrade_request(&to_upgrade) {
                            None => PendingUpgrade::default_upgrade(to_upgrade)?,
                            Some(request) => PendingUpgrade::Missing(to_upgrade, gdcf.refresh(&request)),
                        },
                },

            // Outdated entry
            ProcessRequestFutureState::Outdated(CacheEntry::Cached(request_result, _), refresh_future) => {
                let upgrade = U::lookup_upgrade(&to_upgrade, &cache, request_result).map_err(Error::Cache)?;

                PendingUpgrade::Outdated(to_upgrade, upgrade, refresh_future)
            },

            _ => unimplemented!(),
        };

        Ok(mode)
    }
}
