use futures::{Async, Future};
use log::{debug, error, trace, warn};

use crate::{
    api::{client::MakeRequest, request::Request, ApiClient},
    cache::{Cache, CacheEntry, CanCache, CreatorKey, Lookup, NewgroundsSongKey, Store},
    error::Error,
    future::{refresh::RefreshCacheFuture, stream::GdcfStream, PeekableFuture, StreamableFuture},
    upgrade::{Upgradable, UpgradeQueryFuture},
    Gdcf,
};
use failure::_core::fmt::Formatter;
use std::fmt::Debug;

struct PendingUpgrade<A, C, Into, U>
where
    A: MakeRequest<U::Request>,
    C: Cache + Store<CreatorKey> + Store<NewgroundsSongKey> + CanCache<U::Request>,
    U: Upgradable<Into>,
{
    to_upgrade: U,
    cache_meta: C::CacheEntryMeta,
    upgrade_future: UpgradeQueryFuture<RefreshCacheFuture<U::Request, A, C>, U::Upgrade>,
}

impl<A, C, Into, U> Debug for PendingUpgrade<A, C, Into, U>
where
    A: MakeRequest<U::Request>,
    C: Cache + Store<CreatorKey> + Store<NewgroundsSongKey> + CanCache<U::Request>,
    U: Upgradable<Into> + Debug,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("PendingUpgrade")
            .field("to_upgrade", &self.to_upgrade)
            .field("cache_meta", &self.cache_meta)
            .field("upgrade_future", &self.upgrade_future)
            .finish()
    }
}

pub struct UpgradeFuture<A, C, From, Into, U>
where
    A: ApiClient + MakeRequest<U::Request>,
    C: Cache + CanCache<U::Request> + CanCache<CreatorKey> + CanCache<NewgroundsSongKey> + Lookup<U::LookupKey>,
    From: PeekableFuture<Item = CacheEntry<U, C::CacheEntryMeta>, Error = Error<A::Err, C::Err>>,
    U: Upgradable<Into>,
{
    gdcf: Gdcf<A, C>,
    forced_refresh: bool,
    inner_future: From,
    pending_upgrade: Option<PendingUpgrade<A, C, Into, U>>, //state: UpgradeFutureState<A, C, From, Into, U>,
}

impl<A, C, From, Into, U> UpgradeFuture<A, C, From, Into, U>
where
    A: ApiClient + MakeRequest<U::Request>,
    C: Cache + CanCache<U::Request> + CanCache<CreatorKey> + CanCache<NewgroundsSongKey> + Lookup<U::LookupKey>,
    From: PeekableFuture<Item = CacheEntry<U, C::CacheEntryMeta>, Error = Error<A::Err, C::Err>> + StreamableFuture<A, C>,
    U: Upgradable<Into>,
{
    pub fn stream(self) -> GdcfStream<A, C, Self> {
        GdcfStream::new(self.gdcf.clone(), self)
    }
}

impl<A, C, From, Into, U> UpgradeFuture<A, C, From, Into, U>
where
    A: ApiClient + MakeRequest<U::Request>,
    C: Cache + CanCache<U::Request> + CanCache<CreatorKey> + CanCache<NewgroundsSongKey> + Lookup<U::LookupKey>,
    From: PeekableFuture<Item = CacheEntry<U, C::CacheEntryMeta>, Error = Error<A::Err, C::Err>>,
    U: Upgradable<Into>,
{
    pub(crate) fn new(gdcf: Gdcf<A, C>, forced_refresh: bool, inner_future: From) -> Self {
        Self {
            gdcf,
            inner_future,
            forced_refresh,
            pending_upgrade: None,
        }
    }
}
impl<A, C, From, Into, U> StreamableFuture<A, C> for UpgradeFuture<A, C, From, Into, U>
where
    A: ApiClient + MakeRequest<U::Request>,
    C: Cache + CanCache<U::Request> + CanCache<CreatorKey> + CanCache<NewgroundsSongKey> + Lookup<U::LookupKey>,
    From: PeekableFuture<Item = CacheEntry<U, C::CacheEntryMeta>, Error = Error<A::Err, C::Err>> + StreamableFuture<A, C>,
    U: Upgradable<Into>,
{
    fn next(self, gdcf: &Gdcf<A, C>) -> Result<Self, Self::Error> {
        Ok(Self {
            inner_future: self.inner_future.next(gdcf)?,
            pending_upgrade: None,
            ..self
        })
    }
}

impl<A, C, From, Into, U> UpgradeFuture<A, C, From, Into, U>
where
    A: ApiClient + MakeRequest<U::Request>,
    C: Cache + CanCache<U::Request> + CanCache<CreatorKey> + CanCache<NewgroundsSongKey> + Lookup<U::LookupKey>,
    From: PeekableFuture<Item = CacheEntry<U, C::CacheEntryMeta>, Error = Error<A::Err, C::Err>>,
    U: Upgradable<Into>,
{
    pub fn upgrade<Into2>(self) -> UpgradeFuture<A, C, Self, Into2, Into>
    where
        Into: Upgradable<Into2>,
        A: MakeRequest<Into::Request>,
        C: CanCache<Into::Request> + Lookup<Into::LookupKey>,
    {
        UpgradeFuture {
            forced_refresh: self.forced_refresh,
            gdcf: self.gdcf.clone(),
            inner_future: self,
            pending_upgrade: None,
        }
    }
}
impl<A, C, From, Into, U> UpgradeFuture<A, C, From, Vec<Into>, Vec<U>>
where
    A: ApiClient + MakeRequest<U::Request>,
    C: Cache + CanCache<U::Request> + CanCache<CreatorKey> + CanCache<NewgroundsSongKey> + Lookup<U::LookupKey>,
    From: PeekableFuture<Item = CacheEntry<Vec<U>, C::CacheEntryMeta>, Error = Error<A::Err, C::Err>>,
    U: Upgradable<Into>,
{
    pub fn upgrade_all<Into2>(self) -> UpgradeFuture<A, C, Self, Vec<Into2>, Vec<Into>>
    where
        Into: Upgradable<Into2>,
        A: MakeRequest<Into::Request>,
        C: Lookup<Into::LookupKey> + CanCache<Into::Request>,
    {
        UpgradeFuture {
            forced_refresh: self.forced_refresh,
            gdcf: self.gdcf.clone(),
            inner_future: self,
            pending_upgrade: None,
        }
    }
}

impl<A, C, From, Into, U> Future for UpgradeFuture<A, C, From, Into, U>
where
    A: ApiClient + MakeRequest<U::Request>,
    C: Cache + CanCache<U::Request> + CanCache<CreatorKey> + CanCache<NewgroundsSongKey> + Lookup<U::LookupKey>,
    From: PeekableFuture<Item = CacheEntry<U, C::CacheEntryMeta>, Error = Error<A::Err, C::Err>>,
    U: Upgradable<Into>,
{
    type Error = From::Error;
    type Item = CacheEntry<Into, C::CacheEntryMeta>;

    fn poll(&mut self) -> Result<Async<Self::Item>, Self::Error> {
        if self.pending_upgrade.is_none() {
            match self.inner_future.poll()? {
                Async::Ready(CacheEntry::Cached(to_upgrade, cache_meta)) => {
                    let upgrade_query = to_upgrade.query_upgrade(&self.gdcf.cache(), self.forced_refresh)?;

                    self.pending_upgrade = Some(PendingUpgrade {
                        to_upgrade,
                        cache_meta,
                        upgrade_future: upgrade_query.futurize(&self.gdcf),
                    });
                },
                Async::Ready(cache_entry) => return Ok(Async::Ready(cache_entry.map_empty())),
                Async::NotReady => return Ok(Async::NotReady),
            }
        }

        if let Some(ref mut pending_upgrade) = self.pending_upgrade {
            match pending_upgrade.upgrade_future.poll()? {
                Async::NotReady => Ok(Async::NotReady),
                Async::Ready(upgrade_query) => {
                    let pending = self.pending_upgrade.take().unwrap();

                    let upgrades = pending.to_upgrade.process_query_result(&self.gdcf.cache(), upgrade_query)?;
                    let upgraded = pending.to_upgrade.upgrade(upgrades).0;

                    Ok(Async::Ready(CacheEntry::Cached(upgraded, pending.cache_meta)))
                },
            }
        } else {
            Ok(Async::NotReady)
        }
    }
}

impl<A, C, From, Into, U> PeekableFuture for UpgradeFuture<A, C, From, Into, U>
where
    A: ApiClient + MakeRequest<U::Request>,
    C: Cache + CanCache<U::Request> + CanCache<CreatorKey> + CanCache<NewgroundsSongKey> + Lookup<U::LookupKey>,
    From: PeekableFuture<Item = CacheEntry<U, C::CacheEntryMeta>, Error = Error<A::Err, C::Err>>,
    U: Upgradable<Into>,
{
    fn peek<F: FnOnce(Self::Item) -> Result<Self::Item, Self::Error>>(mut self, f: F) -> Result<Self, Self::Error> {
        // FIXME: this only requires &mut self access. maybe that's enough always?
        match self.pending_upgrade {
            None => {
                let cache = self.gdcf.cache(); // do not borrow self into the closure

                self.inner_future = self.inner_future.peek(|cache_entry| {
                    if let CacheEntry::Cached(to_upgrade, meta) = cache_entry {
                        let upgrade_query = to_upgrade.query_upgrade(&cache, false)?;

                        if upgrade_query.upgrade_cached() {
                            let (upgraded, downgrades) = to_upgrade.upgrade(upgrade_query);

                            if let CacheEntry::Cached(upgraded, meta) = f(CacheEntry::Cached(upgraded, meta))? {
                                Ok(CacheEntry::Cached(U::downgrade(upgraded, downgrades).0, meta))
                            } else {
                                panic!("function passed to .peek() mutated cache entry in invalid ways")
                            }
                        } else {
                            Ok(CacheEntry::Cached(to_upgrade, meta))
                        }
                    } else {
                        Ok(f(cache_entry.map_empty())?.map_empty())
                    }
                })?;
            },
            Some(_) => {
                let pending_upgrade = self.pending_upgrade.take().unwrap();

                let (futures, upgrades) = pending_upgrade.upgrade_future.mitosis();
                let (upgraded, downgrades) = pending_upgrade.to_upgrade.upgrade(upgrades);

                if let CacheEntry::Cached(upgraded, cache_meta) = f(CacheEntry::Cached(upgraded, pending_upgrade.cache_meta))? {
                    let (to_upgrade, upgrades) = U::downgrade(upgraded, downgrades);

                    self.pending_upgrade = Some(PendingUpgrade {
                        to_upgrade,
                        cache_meta,
                        upgrade_future: futures.recombination(upgrades),
                    });
                } else {
                    panic!("function passed to .peek() mutated cache entry in invalid ways")
                }
            },
        };

        Ok(self)
    }
}

impl<A, C, From, Into, U> Debug for UpgradeFuture<A, C, From, Into, U>
where
    A: ApiClient + MakeRequest<U::Request>,
    C: Cache + CanCache<U::Request> + CanCache<CreatorKey> + CanCache<NewgroundsSongKey> + Lookup<U::LookupKey>,
    From: PeekableFuture<Item = CacheEntry<U, C::CacheEntryMeta>, Error = Error<A::Err, C::Err>> + Debug,
    U: Upgradable<Into> + Debug,
{
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        f.debug_struct("UpgradeFuture")
            .field("forced_refresh", &self.forced_refresh)
            .field("inner_future", &self.inner_future)
            .field("pending_upgrade", &self.pending_upgrade)
            .finish()
    }
}
