//! Module containing the machinery GDCF internally uses to upgrade requests and objects

use crate::{
    api::{client::MakeRequest, request::Request},
    cache::{Cache, CacheEntry, CanCache, CreatorKey, Key, Lookup, NewgroundsSongKey, Store},
    error::{ApiError, CacheError, Error},
    future::refresh::RefreshCacheFuture,
    Gdcf,
};
use futures::{Async, Future};
use std::fmt::Debug;

pub mod level;
pub mod user;

#[derive(Debug)]
pub enum UpgradeQuery<R, S> {
    One(Option<R>, Option<S>),
    Many(Vec<UpgradeQuery<R, S>>),
}

impl<R, S> UpgradeQuery<R, S> {
    fn clone_upgrades(&self) -> UpgradeQuery<(), S>
    where
        S: Clone,
    {
        match self {
            UpgradeQuery::One(_, s) => UpgradeQuery::One(None, s.clone()),
            UpgradeQuery::Many(inner_queries) => UpgradeQuery::Many(inner_queries.iter().map(UpgradeQuery::clone_upgrades).collect()),
        }
    }

    fn one(self) -> (Option<R>, Option<S>) {
        match self {
            UpgradeQuery::One(r, s) => (r, s),
            UpgradeQuery::Many(_) => panic!("Expected UpgradeQuery::One"),
        }
    }

    pub(crate) fn upgrade_cached(&self) -> bool {
        match self {
            UpgradeQuery::One(_, upgrade) => upgrade.is_some(),
            UpgradeQuery::Many(inner_queries) => inner_queries.iter().all(|query| query.upgrade_cached()),
        }
    }

    fn mitosis(self) -> (UpgradeQuery<R, ()>, UpgradeQuery<(), S>) {
        match self {
            UpgradeQuery::One(left, right) => (UpgradeQuery::One(left, None), UpgradeQuery::One(None, right)),
            UpgradeQuery::Many(inner_queries) => {
                let mut lefts = Vec::new();
                let mut rights = Vec::new();

                for inner_query in inner_queries {
                    let (left, right) = inner_query.mitosis();

                    lefts.push(left);
                    rights.push(right);
                }

                (UpgradeQuery::Many(lefts), UpgradeQuery::Many(rights))
            },
        }
    }
}

impl<R> UpgradeQuery<R, ()> {
    fn recombination<S>(self, other: UpgradeQuery<(), S>) -> UpgradeQuery<R, S> {
        match (self, other) {
            (UpgradeQuery::One(left, _), UpgradeQuery::One(_, right)) => UpgradeQuery::One(left, right),
            (UpgradeQuery::Many(lefts), UpgradeQuery::Many(rights)) =>
                UpgradeQuery::Many(
                    lefts
                        .into_iter()
                        .zip(rights)
                        .map(|(left, right)| left.recombination(right))
                        .collect(),
                ),
            _ => panic!("Invalid recombination paramers. Can only combine when both upgrade query objects have the same structure"),
        }
    }
}

impl<R: Request, S> UpgradeQuery<R, S> {
    pub(crate) fn futurize<A, C>(self, gdcf: &Gdcf<A, C>) -> UpgradeQueryFuture<RefreshCacheFuture<R, A, C>, S>
    where
        A: MakeRequest<R>,
        C: Cache + CanCache<R> + Store<CreatorKey> + Store<NewgroundsSongKey>,
    {
        match self {
            UpgradeQuery::One(request, data) =>
                UpgradeQueryFuture::One(request.map(|req| FutureState::Pending(RefreshCacheFuture::new(gdcf, req))), data),
            UpgradeQuery::Many(inner) =>
                UpgradeQueryFuture::Many(
                    inner
                        .into_iter()
                        .map(|inner_query| FutureState::Pending(inner_query.futurize(gdcf)))
                        .collect(),
                ),
        }
    }
}

#[derive(Debug)]
pub(crate) enum FutureState<F: Future> {
    Pending(F),
    Done(F::Item),
}

pub(crate) enum UpgradeQueryFuture<F: Future, S> {
    One(Option<FutureState<F>>, Option<S>),
    Many(Vec<FutureState<UpgradeQueryFuture<F, S>>>),
}

impl<F: Future, S> Debug for UpgradeQueryFuture<F, S>
where
    F: Debug,
    F::Item: Debug,
    S: Debug,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            UpgradeQueryFuture::One(fut, s) => f.debug_tuple("One").field(fut).field(s).finish(),
            UpgradeQueryFuture::Many(inners) => f.debug_tuple("Many").field(inners).finish(),
        }
    }
}

impl<F: Future> UpgradeQueryFuture<F, ()> {
    pub(crate) fn recombination<S>(self, other: UpgradeQuery<(), S>) -> UpgradeQueryFuture<F, S> {
        match (self, other) {
            (UpgradeQueryFuture::One(left, _), UpgradeQuery::One(_, right)) => UpgradeQueryFuture::One(left, right),
            (UpgradeQueryFuture::Many(lefts), UpgradeQuery::Many(rights)) =>
                UpgradeQueryFuture::Many(
                    lefts
                        .into_iter()
                        .zip(rights)
                        .map(|(left, right)| {
                            match left {
                                FutureState::Pending(future) => FutureState::Pending(future.recombination(right)),
                                FutureState::Done(upgrade_query) => FutureState::Done(upgrade_query.recombination(right)),
                            }
                        })
                        .collect(),
                ),
            _ => panic!("Invalid recombination parameters. Can only combine when both upgrade query objects have the same structure"),
        }
    }
}

impl<F: Future, S> UpgradeQueryFuture<F, S> {
    pub(crate) fn clone_upgrades(&self) -> UpgradeQuery<(), S>
    where
        S: Clone,
    {
        match self {
            UpgradeQueryFuture::One(_, s) => UpgradeQuery::One(None, s.clone()),
            UpgradeQueryFuture::Many(inner_queries) =>
                UpgradeQuery::Many(
                    inner_queries
                        .iter()
                        .map(|state| {
                            match state {
                                FutureState::Pending(upgrade_future) => upgrade_future.clone_upgrades(),
                                FutureState::Done(upgrade_query) => upgrade_query.clone_upgrades(),
                            }
                        })
                        .collect(),
                ),
        }
    }

    pub(crate) fn mitosis(self) -> (UpgradeQueryFuture<F, ()>, UpgradeQuery<(), S>) {
        match self {
            UpgradeQueryFuture::One(future, data) => (UpgradeQueryFuture::One(future, None), UpgradeQuery::One(None, data)),
            UpgradeQueryFuture::Many(inner_futures) => {
                let mut futures = Vec::new();
                let mut upgrades = Vec::new();

                for future_state in inner_futures {
                    let (future, upgrade) = match future_state {
                        FutureState::Pending(upgrade_future) => {
                            let (future, upgrade) = upgrade_future.mitosis();

                            (FutureState::Pending(future), upgrade)
                        },
                        FutureState::Done(upgrade_query) => {
                            let (lefts, rights) = upgrade_query.mitosis();

                            (FutureState::Done(lefts), rights)
                        },
                    };

                    futures.push(future);
                    upgrades.push(upgrade);
                }

                (UpgradeQueryFuture::Many(futures), UpgradeQuery::Many(upgrades))
            },
        }
    }
}

impl<F: Future, S> Future for UpgradeQueryFuture<F, S> {
    type Error = F::Error;
    type Item = UpgradeQuery<F::Item, S>;

    fn poll(&mut self) -> Result<Async<Self::Item>, Self::Error> {
        match self {
            UpgradeQueryFuture::One(Some(FutureState::Pending(future)), data) =>
                match future.poll()? {
                    Async::Ready(future_result) => Ok(Async::Ready(UpgradeQuery::One(Some(future_result), data.take()))),
                    Async::NotReady => Ok(Async::NotReady),
                },
            UpgradeQueryFuture::One(Some(FutureState::Done(_)), _) => unreachable!(), /* can be constructed, but we don't poll this */
            // anymore! (see below)
            UpgradeQueryFuture::One(None, data) => Ok(Async::Ready(UpgradeQuery::One(None, data.take()))),
            UpgradeQueryFuture::Many(inner) => {
                let mut all_done = true;

                for i in 0..inner.len() {
                    match &mut inner[i] {
                        FutureState::Pending(future) =>
                            match future.poll()? {
                                Async::NotReady => {
                                    all_done = false;
                                },
                                Async::Ready(done) => inner[i] = FutureState::Done(done),
                            },
                        FutureState::Done(_) => (), // no polling here
                    }
                }

                if all_done {
                    log::debug!("All requests of upgrade query future done!");

                    Ok(Async::Ready(UpgradeQuery::Many(
                        std::mem::replace(inner, Vec::new())
                            .into_iter()
                            .map(|future_state| {
                                match future_state {
                                    FutureState::Pending(_) => unreachable!(),
                                    FutureState::Done(yes) => yes,
                                }
                            })
                            .collect(),
                    )))
                } else {
                    Ok(Async::NotReady)
                }
            },
        }
    }
}

#[derive(Debug)]
pub enum UpgradeError<E: CacheError> {
    UpgradeFailed,
    Cache(E),
}

impl<A: ApiError, C: CacheError> From<UpgradeError<C>> for Error<A, C> {
    fn from(err: UpgradeError<C>) -> Self {
        match err {
            UpgradeError::UpgradeFailed => Error::UnexpectedlyAbsent,
            UpgradeError::Cache(cache_error) => Error::Cache(cache_error),
        }
    }
}

impl<C: CacheError> From<C> for UpgradeError<C> {
    fn from(cache_error: C) -> Self {
        UpgradeError::Cache(cache_error)
    }
}

/// Trait for upgrading objects
///
/// Implementing this trait for some type means that instances of that type can be upgraded into
/// instances of type `Into`.
///
/// Upgrading can either be realised by upgrading some component of an object (for instance when
/// upgrading the id of a song into their [`NewgroundsSong`] objects in a [`PartialLevel`]), or by
/// replacing the whole object alltogether (for instance when upgrading a [`SearchedUser`] into a
/// profile.
///
/// Upgrading does not perform any cloning.
pub trait Upgradable<Into>: Sized {
    /// The part of the object that's being upgraded. If the whole object is upgraded, this should
    /// be [`Self`]
    type From;

    /// The request that has to be made for the upgrade to work
    type Request: Request;

    /// The object [`Self::From`] is being replaced by. If the whole object is upgraded, this should
    /// be `Into`
    type Upgrade;

    /// If applicable, the key of the object that has to be looked up in the cache to perform an
    /// upgrade.
    ///
    /// If no lookup beyond one of [`Upgrdable::Request`] is required, set this to the never type or
    /// `Upgradable::Request`.
    type LookupKey: Key;

    /// Determines how this upgrade has to be done by either producing the request that needs to be
    /// made to retrieve the data needed, or returning the [`Upgradable::Upgrade`] object.
    ///
    /// Note that it is not enough to just return the request here and have GDCF check if the result
    /// of that request has already been cached, as certain objects can be retrieved by different
    /// requests. For instance, the song for a level could already be cached because another level
    /// in cache uses the same song, yet the request to retrieve the song (a [`LevelsRequest`]) has
    /// never been made.
    ///
    /// ## Parameters:
    /// + `cache`: The cache to look in
    /// + `ignore_cached`: Whether cached data should be ignored, meaning that requests should be
    /// produced for all upgrade data, no matter whether its cached or not
    ///
    /// ## A note on the return value
    /// Implementations of this method have to handle the cache appropriately themselves. This means
    /// in particular, that they have to handle outdated cache entries by returning both an upgrade
    /// and a request!
    ///
    /// Furthermore, a [`UpgradeQuery`] with both fields set to [`None`] must never be constructed!
    fn query_upgrade<C: Cache + Lookup<Self::LookupKey>>(
        &self,
        cache: &C,
        ignored_cached: bool,
    ) -> Result<UpgradeQuery<Self::Request, Self::Upgrade>, UpgradeError<C::Err>>;

    fn process_query_result<C: Cache + Lookup<Self::LookupKey>>(
        &self,
        cache: &C,
        resolved_query: UpgradeQuery<CacheEntry<<Self::Request as Request>::Result, C::CacheEntryMeta>, Self::Upgrade>,
    ) -> Result<UpgradeQuery<(), Self::Upgrade>, UpgradeError<C::Err>>;

    fn upgrade<State>(self, upgrade: UpgradeQuery<State, Self::Upgrade>) -> (Into, UpgradeQuery<State, Self::From>);
    fn downgrade<State>(upgraded: Into, downgrade: UpgradeQuery<State, Self::From>) -> (Self, UpgradeQuery<State, Self::Upgrade>);
}

impl<Into, U> Upgradable<Vec<Into>> for Vec<U>
where
    U: Upgradable<Into>,
{
    type From = U::From;
    type LookupKey = U::LookupKey;
    type Request = U::Request;
    type Upgrade = U::Upgrade;

    fn query_upgrade<C: Cache + Lookup<Self::LookupKey>>(
        &self,
        cache: &C,
        ignore_cached: bool,
    ) -> Result<UpgradeQuery<Self::Request, Self::Upgrade>, UpgradeError<C::Err>> {
        // Alright, so we have to jump through some hoops to get this work sadly. ApiClients can process
        // lists of requests, however we still need to keep track of which request corresponds to which
        // element to upgrade. We store this information in the upgrade list, which will contain a [`None`]
        // entry whenever an upgrade needed a request and we rely on the invariant that neither the upgrades
        // vector, nor the request vector, gets reordered.
        let mut queries = Vec::new();

        for to_query in self.iter() {
            let query = to_query.query_upgrade(cache, ignore_cached)?;

            queries.push(query);
        }

        Ok(UpgradeQuery::Many(queries))
    }

    fn process_query_result<C: Cache + Lookup<Self::LookupKey>>(
        &self,
        cache: &C,
        resolved_query: UpgradeQuery<CacheEntry<<Self::Request as Request>::Result, C::CacheEntryMeta>, Self::Upgrade>,
    ) -> Result<UpgradeQuery<(), Self::Upgrade>, UpgradeError<C::Err>> {
        match resolved_query {
            UpgradeQuery::One(..) => panic!(),
            UpgradeQuery::Many(inner_queries) =>
                Ok(UpgradeQuery::Many(
                    self.iter()
                        .zip(inner_queries)
                        .map(|(result, query)| result.process_query_result(cache, query))
                        .collect::<Result<_, _>>()?,
                )),
        }
    }

    fn upgrade<State>(self, upgrade: UpgradeQuery<State, Self::Upgrade>) -> (Vec<Into>, UpgradeQuery<State, Self::From>) {
        if let UpgradeQuery::Many(upgrades) = upgrade {
            let mut upgraded = Vec::new();
            let mut downgrades = Vec::new();

            for (to_upgrade, upgrade) in self.into_iter().zip(upgrades) {
                let (to_downgrade, downgrade) = to_upgrade.upgrade(upgrade);

                upgraded.push(to_downgrade);
                downgrades.push(downgrade);
            }

            (upgraded, UpgradeQuery::Many(downgrades))
        } else {
            panic!("Attempt to upgrade list of upgradables with a single upgrade")
        }
    }

    fn downgrade<State>(upgraded: Vec<Into>, downgrade: UpgradeQuery<State, Self::From>) -> (Self, UpgradeQuery<State, Self::Upgrade>) {
        if let UpgradeQuery::Many(downgrades) = downgrade {
            let mut downgraded = Vec::new();
            let mut upgrades = Vec::new();

            for (to_downgrade, downgrade) in upgraded.into_iter().zip(downgrades) {
                let (to_upgrade, upgrade) = U::downgrade(to_downgrade, downgrade);

                downgraded.push(to_upgrade);
                upgrades.push(upgrade);
            }

            (downgraded, UpgradeQuery::Many(upgrades))
        } else {
            panic!("Attempt to downgrade list of upgradables with a single downgrade")
        }
    }
}
