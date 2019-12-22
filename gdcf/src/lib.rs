#![deny(
    bare_trait_objects,
    missing_debug_implementations,
    unused_extern_crates,
    patterns_in_fns_without_body,
    stable_features,
    unknown_lints,
    unused_features,
    unused_imports,
    unused_parens
)]

use crate::{
    api::{
        client::MakeRequest,
        request::{comment::ProfileCommentsRequest, user::UserSearchRequest, LevelRequest, LevelsRequest, Request, UserRequest},
        ApiClient,
    },
    cache::{Cache, CacheEntry, CanCache, CreatorKey, NewgroundsSongKey, Store},
    future::{
        process::{ProcessRequestFuture, ProcessRequestFutureState},
        refresh::RefreshCacheFuture,
    },
};
pub use error::Error;
use gdcf_model::{song::NewgroundsSong, user::Creator};
use log::{info, trace};

#[macro_use]
mod macros;

pub mod api;
pub mod cache;
pub mod error;
pub mod future;
pub mod upgrade;

// FIXME: move this somewhere more fitting
#[derive(Debug, Clone, PartialEq)]
pub enum Secondary {
    NewgroundsSong(NewgroundsSong),
    Creator(Creator),
    MissingCreator(u64),
    MissingNewgroundsSong(u64),
}

impl From<NewgroundsSong> for Secondary {
    fn from(song: NewgroundsSong) -> Self {
        Secondary::NewgroundsSong(song)
    }
}

impl From<Creator> for Secondary {
    fn from(creator: Creator) -> Self {
        Secondary::Creator(creator)
    }
}

impl std::fmt::Display for Secondary {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Secondary::NewgroundsSong(inner) => inner.fmt(f),
            Secondary::Creator(inner) => inner.fmt(f),
            Secondary::MissingCreator(cid) => write!(f, "Creator object missing server-sided: {}", cid),
            Secondary::MissingNewgroundsSong(nid) => write!(f, "Newgrounds song object missing server-sided: {}", nid),
        }
    }
}

#[derive(Debug, Clone)]
pub struct Gdcf<A, C>
where
    A: ApiClient,
    C: Cache,
{
    client: A,
    cache: C,
}

impl<A, C> Gdcf<A, C>
where
    A: ApiClient,
    C: Cache,
{
    pub fn new(client: A, cache: C) -> Gdcf<A, C> {
        Gdcf { client, cache }
    }

    pub fn cache(&self) -> C {
        self.cache.clone()
    }

    pub fn client(&self) -> A {
        self.client.clone()
    }
}

impl<A, C> Gdcf<A, C>
where
    A: ApiClient,
    C: Cache + Store<CreatorKey> + Store<NewgroundsSongKey>,
{
    fn process<R>(&self, request: R, force_refresh: bool) -> Result<ProcessRequestFutureState<R, A, C>, C::Err>
    where
        R: Request,
        A: MakeRequest<R>,
        C: CanCache<R>,
    {
        info!("Processing request {:?}", request);

        let cached = match self.cache.lookup(&request)? {
            CacheEntry::Missing => {
                info!("No cache entry for request {:?}", request);

                None
            },
            entry =>
                if entry.is_expired() {
                    trace!("Cache entry is {:?}", entry);
                    info!("Cache entry for request {:?} is expired!", request);

                    Some(entry)
                } else if force_refresh {
                    trace!("Cache entry is {:?}", entry);
                    info!("Cache entry is up-to-date, but request forces refresh!");

                    Some(entry)
                } else {
                    trace!("Cache entry is {:?}", entry);
                    info!("Cached entry for request {:?} is up-to-date!", request);

                    return Ok(ProcessRequestFutureState::UpToDate(Some(entry), request))
                },
        };

        let future = RefreshCacheFuture::new(self, request);

        Ok(match cached {
            Some(value) => ProcessRequestFutureState::Outdated(value, future),
            None => ProcessRequestFutureState::Uncached(future),
        })
    }
}

impl<A, C> Gdcf<A, C>
where
    A: ApiClient,
    C: Cache + Store<NewgroundsSongKey> + Store<CreatorKey>,
{
    /// Processes the given [`LevelRequest`]
    ///
    /// The `User` and `Song` type parameters determine, which sequence of requests should be made
    /// to retrieve the [`Level`]. A plain request to `downloadGJLevel` is equivalent to a call of
    /// `Gdcf::level<u64, u64>`
    ///
    /// `User` can currently be one of the following:
    /// + [`u64`] - The creator is provided as his user ID. Causes no additional requests.
    /// + [`Creator`] - Causes an additional [`LevelsRequest`] to retrieve the creator.
    /// + [`User`] - Causes an additional [`UserRequest`]  to retrieve the creator's profile (Not
    /// Yet Implemented)
    ///
    /// `Song` can currently be one of the following:
    /// + [`u64`] - The custom song is provided only as its newgrounds ID. Causes no additional
    /// requests
    /// + [`NewgroundsSong`] - Causes an additional [`LevelsRequest`] to be made to
    /// retrieve the custom song (only if the level actually uses a custom song though)
    ///
    /// Note that a call of `Gdcf::level<NewgroundsSong, Creator>` will **not** issue the same
    /// `LevelsRequest` twice - GDCF will recognize the cache to be up-to-date when it attempts the
    /// second one and uses the cached value (or at least it will if you set cache-expiry to
    /// anything larger than 0 seconds - but then again why would you use GDCF if you don't use the
    /// cache)
    pub fn level(&self, request: impl Into<LevelRequest>, force_refresh: bool) -> Result<ProcessRequestFuture<LevelRequest, A, C>, C::Err>
    where
        A: MakeRequest<LevelRequest>,
        C: CanCache<LevelRequest>,
    {
        ProcessRequestFuture::new(self.clone(), request.into(), force_refresh)
    }

    /// Processes the given [`LevelsRequest`]
    ///
    /// The `User` and `Song` type parameters determine, which sequence of requests should be made
    /// to retrieve the [`Level`].
    ///
    /// `User` can currently be one of the following:
    /// + [`u64`] - The creator are only provided as their user IDs. Causes no additional requests
    /// + [`Creator`] - Causes no additional requests
    /// + [`User`] - Causes up to 10 additional [`UserRequest`]s to retrieve every creator's
    /// profile
    ///
    /// `Song` can currently be one of the following:
    /// + [`u64`] - The custom song is provided only as its newgrounds ID. Causes no additional
    /// requests
    /// + [`NewgroundsSong`] - Causes no additional requests.
    pub fn levels(
        &self,
        request: impl Into<LevelsRequest>,
        force_refresh: bool,
    ) -> Result<ProcessRequestFuture<LevelsRequest, A, C>, C::Err>
    where
        A: MakeRequest<LevelsRequest>,
        C: CanCache<LevelsRequest>,
    {
        ProcessRequestFuture::new(self.clone(), request.into(), force_refresh)
    }

    /// Processes the given [`UserRequest`]
    pub fn user(&self, request: impl Into<UserRequest>, force_refresh: bool) -> Result<ProcessRequestFuture<UserRequest, A, C>, C::Err>
    where
        A: MakeRequest<UserRequest>,
        C: CanCache<UserRequest>,
    {
        ProcessRequestFuture::new(self.clone(), request.into(), force_refresh)
    }

    pub fn search_user(
        &self,
        request: impl Into<UserSearchRequest>,
        force_refresh: bool,
    ) -> Result<ProcessRequestFuture<UserSearchRequest, A, C>, C::Err>
    where
        A: MakeRequest<UserSearchRequest>,
        C: CanCache<UserSearchRequest>,
    {
        ProcessRequestFuture::new(self.clone(), request.into(), force_refresh)
    }

    pub fn profile_comments(
        &self,
        request: impl Into<ProfileCommentsRequest>,
        force_refresh: bool,
    ) -> Result<ProcessRequestFuture<ProfileCommentsRequest, A, C>, C::Err>
    where
        A: MakeRequest<ProfileCommentsRequest>,
        C: CanCache<ProfileCommentsRequest>,
    {
        ProcessRequestFuture::new(self.clone(), request.into(), force_refresh)
    }
}
