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

//! The `gdcf` crate is the core of the Geometry Dash Caching Framework.
//! It provides all the core traits required to implement an API Client and
//! a cache which are used by [`Gdcf`].
//!
//! # Geometry Dash Caching Framework
//!
//! The idea behind the Geometry Dash Caching Framework is to provide fast and
//! reliable access to the resources provided by the Geometry Dash servers. It
//! achieves this goal by caching all responses from the servers. When a resource is requested, it
//! is first looked up in the cache. If the cache entry is not yet expired, it is simply returned
//! and the request can be handled nearly instantly without any interaction with the Geometry Dash
//! servers. If the cache entry is existing, but expired, GDCF will make an asynchronous request to
//! the Geometry Dash servers and create a [Future](GdcfFuture) that resolves to the result of that
//! request, while also providing access to the cached value (without the need to poll the Future
//! to completion). The only time you are actually forced to wait for a response from the Geometry
//! Dash servers is when the cache entry for a request isn't existing.
//!
//! Further, GDCF has the ability to "glue together" multiple requests to provide more information
//! about requested objects. It is, for example, possible to issue a [`LevelRequest`]
//! (`downloadGJLevel`) and have GDCF automatically issue a [`LevelsRequest`] (`getGJLevels`) to
//! retrieve the creator and newgrounds song, which aren't provided by the former endpoint.
//!
//! # How to use:
//! This crate only provides the required traits for caches and API clients, and the code that
//! connects them. To use GDCF you first need to either find yourself an existing implementation of
//! those, or write your own.
//!
//! The following example uses the `gdcf_dbcache` crate as its cache implementation (a database
//! cache with sqlite and postgreSQL backend) and the `gdrs` crate as its API client.
//!
//! ```rust
//! // First we need to configure the cache. Here we're using a sqlite in-memory database
//! // whose cache entries expire after 30 minutes.
//! let mut config = DatabaseCacheConfig::sqlite_memory_config();
//! config.invalidate_after(Duration::minutes(30));
//!
//! // Then we can create the actual cache and API wrapper
//! let cache = DatabaseCache::new(config);
//! let client = BoomlingsClient::new();
//!
//! // A database cache needs to go through initialization before it can be used, as it
//! // needs to create all the required tables
//! cache.initialize()?;
//!
//! // Then we can create an instance of the Gdcf struct, which we will use to
//! // actually make all our requests
//! let gdcf = Gdcf::new(client, cache);
//!
//! // And we're good to go! To make a request, we need to initialize one of the
//! // request structs. Here, we're make a requests to retrieve the 6th page of
//! // featured demon levels of any demon difficulty
//! let request = LevelsRequest::default()
//!     .request_type(LevelRequestType::Featured)
//!     .with_rating(LevelRating::Demon(DemonRating::Hard))
//!     .page(5);
//!
//! // To actually issue the request, we call the appropriate method on our Gdcf instance.
//! // The type parameters on these methods determine how much associated information
//! // should be retrieved for the request result. Here we're telling GDCF to also
//! // get us information about the requested levels' custom songs and creators
//! // instead of just their IDs. "paginate_levels" give us a stream over all pages
//! // of results from our request instead of only the page we requested.
//! let stream = gdcf.paginate_levels::<NewgroundsSong, Creator>(request);
//!
//! // Since we have a stream, we can use all our favorite Stream methods from the
//! // futures crate. Here we limit the stream to 50 pages of levels and print
//! // out each level's name, creator, song and song artist.
//! let future = stream
//!     .take(50)
//!     .for_each(|levels| {
//!         for level in levels {
//!             match level.custom_song {
//!                 Some(newgrounds_song) =>
//!                     println!(
//!                         "Retrieved demon level {} by {} using custom song {} by {}",
//!                         level.name, level.creator.name, newgrounds_song.name, newgrounds_song.artist
//!                     ),
//!                 None =>
//!                     println!(
//!                         "Retrieved demon level {} by {} using main song {} by {}",
//!                         level.name,
//!                         level.creator.name,
//!                         level.main_song.unwrap().name,
//!                         level.main_song.unwrap().artist
//!                     ),
//!             }
//!         }
//!
//!         Ok(())
//!     })
//!     .map_err(|error| eprintln!("Something went wrong! {:?}", error));
//!
//! tokio::run(future);
//! ```

use log::{info, trace};

use gdcf_model::{song::NewgroundsSong, user::Creator};

use crate::{
    api::{
        client::MakeRequest,
        request::{comment::ProfileCommentsRequest, user::UserSearchRequest, LevelRequest, LevelsRequest, Request, UserRequest},
        ApiClient,
    },
    cache::{Cache, CacheEntry, CanCache, Store},
    future::{
        process::{ProcessRequestFuture, ProcessRequestFutureState},
        refresh::RefreshCacheFuture,
        stream::GdcfStream,
        GdcfFuture,
    },
};

pub use error::Error;

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
    C: Cache + Store<Creator> + Store<NewgroundsSong>,
{
    fn refresh<R>(&self, request: &R) -> RefreshCacheFuture<R, A, C>
    where
        R: Request,
        A: MakeRequest<R>,
        C: CanCache<R>,
    {
        info!("Performing refresh on request {}", request);

        RefreshCacheFuture::new(self.clone(), request.key(), self.client().make(&request))
    }

    fn process<R>(&self, request: &R) -> Result<ProcessRequestFutureState<R, A, C>, C::Err>
    where
        R: Request,
        A: MakeRequest<R>,
        C: CanCache<R>,
    {
        info!("Processing request {}", request);

        let cached = match self.cache.lookup_request(&request)? {
            CacheEntry::Missing => {
                info!("No cache entry for request {}", request);

                None
            },
            entry =>
                if entry.is_expired() {
                    trace!("Cache entry is {:?}", entry);
                    info!("Cache entry for request {} is expired!", request);

                    Some(entry)
                } else if request.forces_refresh() {
                    trace!("Cache entry is {:?}", entry);
                    info!("Cache entry is up-to-date, but request forces refresh!");

                    Some(entry)
                } else {
                    trace!("Cache entry is {:?}", entry);
                    info!("Cached entry for request {} is up-to-date!", request);

                    return Ok(ProcessRequestFutureState::UpToDate(entry))
                },
        };

        let future = self.refresh(request);

        Ok(match cached {
            Some(value) => ProcessRequestFutureState::Outdated(value, future),
            None => ProcessRequestFutureState::Uncached(future),
        })
    }
}

impl<A, C> Gdcf<A, C>
where
    A: ApiClient,
    C: Cache + Store<NewgroundsSong> + Store<Creator>,
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
    pub fn level(&self, request: impl Into<LevelRequest>) -> Result<ProcessRequestFuture<LevelRequest, A, C>, C::Err>
    where
        A: MakeRequest<LevelRequest>,
        C: CanCache<LevelRequest>,
    {
        ProcessRequestFuture::new(self.clone(), &request.into())
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
    pub fn levels(&self, request: impl Into<LevelsRequest>) -> Result<ProcessRequestFuture<LevelsRequest, A, C>, C::Err>
    where
        A: MakeRequest<LevelsRequest>,
        C: CanCache<LevelsRequest>,
    {
        ProcessRequestFuture::new(self.clone(), &request.into())
    }

    /// Generates a stream of pages of levels by incrementing the [`LevelsRequest`]'s `page`
    /// parameter until it hits the first empty page.
    pub fn paginate_levels(
        &self,
        request: impl Into<LevelsRequest>,
    ) -> Result<GdcfStream<ProcessRequestFuture<LevelsRequest, A, C>>, C::Err>
    where
        A: MakeRequest<LevelsRequest>,
        C: CanCache<LevelsRequest>,
    {
        GdcfStream::new(self.clone(), request.into())
    }

    /// Processes the given [`UserRequest`]
    pub fn user(&self, request: impl Into<UserRequest>) -> Result<ProcessRequestFuture<UserRequest, A, C>, C::Err>
    where
        A: MakeRequest<UserRequest>,
        C: CanCache<UserRequest>,
    {
        ProcessRequestFuture::new(self.clone(), &request.into())
    }

    pub fn search_user(&self, request: impl Into<UserSearchRequest>) -> Result<ProcessRequestFuture<UserSearchRequest, A, C>, C::Err>
    where
        A: MakeRequest<UserSearchRequest>,
        C: CanCache<UserSearchRequest>,
    {
        ProcessRequestFuture::new(self.clone(), &request.into())
    }

    pub fn profile_comments(
        &self,
        request: impl Into<ProfileCommentsRequest>,
    ) -> Result<ProcessRequestFuture<ProfileCommentsRequest, A, C>, C::Err>
    where
        A: MakeRequest<ProfileCommentsRequest>,
        C: CanCache<ProfileCommentsRequest>,
    {
        ProcessRequestFuture::new(self.clone(), &request.into())
    }

    pub fn paginate_profile_comments(
        &self,
        request: impl Into<ProfileCommentsRequest>,
    ) -> Result<GdcfStream<ProcessRequestFuture<ProfileCommentsRequest, A, C>>, C::Err>
    where
        A: MakeRequest<ProfileCommentsRequest>,
        C: CanCache<ProfileCommentsRequest>,
    {
        GdcfStream::new(self.clone(), request.into())
    }
}
