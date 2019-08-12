#![deny(
    bare_trait_objects,
    missing_debug_implementations,
    unused_extern_crates,
    patterns_in_fns_without_body,
    stable_features,
    unknown_lints,
    unused_features,
    //unused_imports,
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

use crate::{
    api::{
        client::MakeRequest,
        request::{LevelRequest, LevelsRequest, PaginatableRequest, Request, UserRequest},
        ApiClient,
    },
    cache::{Cache, CacheEntry, CanCache, Lookup, Store},
    error::{ApiError, GdcfError},
};
use futures::{future::ok, Future};
use gdcf_model::{
    level::{Level, PartialLevel},
    song::NewgroundsSong,
    user::{Creator, User},
};
use log::{error, info};

pub use crate::future::{GdcfFuture, GdcfStream};
use crate::{
    api::request::{comment::ProfileCommentsRequest, user::UserSearchRequest},
    cache::CacheUserExt,
    future::{extend::ExtendManyFuture, refresh::RefreshCacheFuture, ProcessRequestFuture},
};
use gdcf_model::{comment::ProfileComment, user::SearchedUser};

#[macro_use]
mod macros;

pub mod api;
pub mod cache;
pub mod error;
mod exchange;
pub mod extend;
mod future;

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

pub trait ProcessRequest<A: ApiClient, C: Cache, R: Request, T> {
    type Future: Future<Item = CacheEntry<T, C::CacheEntryMeta>, Error = GdcfError<A::Err, C::Err>>;

    fn process_request(&self, request: R) -> Result<Self::Future, C::Err>;

    // TODO: pagination
}

impl<A, Req, C> ProcessRequest<A, C, Req, Req::Result> for Gdcf<A, C>
where
    Req: Request + Send + Sync + 'static,
    A: ApiClient + MakeRequest<Req>,
    C: Cache + Store<Creator> + Store<NewgroundsSong> + CanCache<Req>,
{
    type Future = ProcessRequestFuture<Req, A, C>;

    fn process_request(&self, request: Req) -> Result<ProcessRequestFuture<Req, A, C>, C::Err> {
        match self.process(request)? {
            EitherOrBoth::A(entry) => Ok(ProcessRequestFuture::UpToDate(entry)),
            EitherOrBoth::B(future) => Ok(ProcessRequestFuture::Uncached(future)),
            EitherOrBoth::Both(entry, future) => Ok(ProcessRequestFuture::Outdated(entry, future)),
        }
    }
}

impl<A, C> ProcessRequest<A, C, LevelsRequest, Vec<PartialLevel<Option<NewgroundsSong>, u64>>> for Gdcf<A, C>
where
    Gdcf<A, C>: ProcessRequest<A, C, LevelsRequest, Vec<PartialLevel<Option<u64>, u64>>>,
    A: ApiClient + MakeRequest<LevelsRequest>,
    C: Cache + CanCache<LevelsRequest> + Store<NewgroundsSong> + Store<Creator> + Lookup<NewgroundsSong>,
{
    type Future = ExtendManyFuture<
        <Gdcf<A, C> as ProcessRequest<A, C, LevelsRequest, Vec<PartialLevel<Option<u64>, u64>>>>::Future,
        A,
        C,
        PartialLevel<Option<NewgroundsSong>, u64>,
        Option<NewgroundsSong>,
        PartialLevel<Option<u64>, u64>,
    >;

    fn process_request(&self, request: LevelsRequest) -> Result<Self::Future, <C as Cache>::Err> {
        Ok(ExtendManyFuture::WaitingOnInner(
            self.clone(),
            request.force_refresh,
            self.process_request(request)?,
        ))
    }
}

pub trait ProcessRequestOld<A: ApiClient, C: Cache, R: Request, T> {
    fn process_request_old(&self, request: R) -> Result<GdcfFuture<T, A::Err, C>, C::Err>;

    fn paginate(&self, request: R) -> Result<GdcfStream<A, C, R, T, Self>, C::Err>
    where
        R: PaginatableRequest,
        Self: Sized + Clone,
    {
        let next = request.next();
        let current = self.process_request_old(request)?;

        Ok(GdcfStream {
            next_request: next,
            current_request: current,
            source: self.clone(),
        })
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

// TODO: eliminate this
enum EitherOrBoth<A, B> {
    A(A),
    B(B),
    Both(A, B),
}

impl<A, C> Gdcf<A, C>
where
    A: ApiClient,
    C: Cache + Store<Creator> + Store<NewgroundsSong>,
{
    fn refresh<R>(&self, request: R) -> RefreshCacheFuture<R, A, C>
    where
        R: Request,
        A: MakeRequest<R>,
        C: CanCache<R>,
    {
        info!("Performing refresh on request {}", request);

        RefreshCacheFuture::new(self.cache(), request.key(), self.client().make(request))
    }

    fn process<R>(&self, request: R) -> Result<EitherOrBoth<CacheEntry<R::Result, C::CacheEntryMeta>, RefreshCacheFuture<R, A, C>>, C::Err>
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
                    info!("Cache entry for request {} is expired!", request);

                    Some(entry)
                } else if request.forces_refresh() {
                    info!("Cache entry is up-to-date, but request forces refresh!");

                    Some(entry)
                } else {
                    info!("Cached entry for request {} is up-to-date!", request);

                    return Ok(EitherOrBoth::A(entry))
                },
        };

        let future = self.refresh(request);

        Ok(match cached {
            Some(value) => EitherOrBoth::Both(value, future),
            None => EitherOrBoth::B(future),
        })
    }
}

impl<A, R, C> ProcessRequestOld<A, C, R, R::Result> for Gdcf<A, C>
where
    R: Request + Send + Sync + 'static,
    A: ApiClient + MakeRequest<R>,
    C: Cache + Store<Creator> + Store<NewgroundsSong> + CanCache<R>,
{
    fn process_request_old(&self, request: R) -> Result<GdcfFuture<R::Result, A::Err, C>, C::Err> {
        match self.process(request)? {
            EitherOrBoth::A(entry) => Ok(GdcfFuture::UpToDate(entry)),
            EitherOrBoth::B(future) => Ok(GdcfFuture::Uncached(Box::new(future))),
            EitherOrBoth::Both(entry, future) => Ok(GdcfFuture::Outdated(entry, Box::new(future))),
        }
    }
}

/*

impl<A, C, Song> ProcessRequest<A, C, LevelRequest, Level<Song, Option<User>>> for Gdcf<A, C>
where
    A: ApiClient + MakeRequest<LevelRequest> + MakeRequest<UserRequest>,
    C: Cache + Store<Creator> + Store<NewgroundsSong> + CanCache<LevelRequest> + CanCache<UserRequest>,
    Song: PartialEq + Send + Clone + 'static,
    Gdcf<A, C>: ProcessRequest<A, C, LevelRequest, Level<Song, Option<Creator>>>,
{
    fn process_request(&self, request: LevelRequest) -> Result<GdcfFuture<Level<Song, Option<User>>, <A as ApiClient>::Err, C>, C::Err> {
        let cache = self.cache();
        let gdcf = self.clone();

        let lookup = move |level: &Level<Song, Option<Creator>>| {
            level
                .base
                .creator
                .as_ref()
                .and_then(|creator| creator.account_id)
                .map(|account_id| cache.lookup(account_id))
                .unwrap_or(Ok(CacheEntry::DeducedAbsent))
        };

        let refresh = move |level: &Level<Song, Option<Creator>>| {
            gdcf.refresh(UserRequest::new(level.base.creator.as_ref().unwrap().account_id.unwrap()))
                .then(|result| {
                    match result {
                        Err(GdcfError::Api(ref err)) if err.is_no_result() => Ok(CacheEntry::DeducedAbsent),
                        Err(err) => Err(err),
                        Ok(thing) => Ok(thing),
                    }
                })
        };

        self.level(request)?.extend(lookup, refresh, exchange::level_user)
    }
}

impl<A, C, Song> ProcessRequest<A, C, LevelsRequest, Vec<PartialLevel<Song, Option<Creator>>>> for Gdcf<A, C>
where
    A: ApiClient + MakeRequest<LevelsRequest>,
    C: Cache + Store<Creator> + Store<NewgroundsSong> + CanCache<LevelsRequest> + Lookup<Creator>,
    Song: PartialEq + Send + Clone + 'static,
    Gdcf<A, C>: ProcessRequest<A, C, LevelsRequest, Vec<PartialLevel<Song, u64>>>,
{
    fn process_request(
        &self,
        request: LevelsRequest,
    ) -> Result<GdcfFuture<Vec<PartialLevel<Song, Option<Creator>>>, <A as ApiClient>::Err, C>, C::Err> {
        let cache = self.cache();

        let lookup = move |level: &PartialLevel<Song, u64>| cache.lookup(level.creator);

        // All creators are provided along with the `LevelsRequest` response. A cache miss above means that
        // the GD servers failed to provide the creator - there's nothing we can do about it, so we just
        // return a future that resolves to `None` here (making a LevelsRequest would obviously lead to an
        // infinite loop of sorts)
        let refresh = move |_: &PartialLevel<Song, u64>| ok(CacheEntry::DeducedAbsent);

        self.levels(request)?
            .extend_all(lookup, refresh, |p, q| Some(exchange::partial_level_user(p, q)))
    }
}

impl<A, C> ProcessRequest<A, C, LevelsRequest, Vec<PartialLevel<NewgroundsSong, u64>>> for Gdcf<A, C>
where
    A: ApiClient + MakeRequest<LevelsRequest>,
    C: Cache + Store<Creator> + Store<NewgroundsSong> + CanCache<LevelsRequest> + Lookup<NewgroundsSong>,
    Gdcf<A, C>: ProcessRequest<A, C, LevelsRequest, Vec<PartialLevel<u64, u64>>>,
{
    fn process_request(
        &self,
        request: LevelsRequest,
    ) -> Result<GdcfFuture<Vec<PartialLevel<NewgroundsSong, u64>>, <A as ApiClient>::Err, C>, C::Err> {
        let cache = self.cache();

        let lookup = move |level: &PartialLevel<u64, u64>| {
            match level.custom_song {
                Some(song_id) => cache.lookup(song_id),
                None => Ok(CacheEntry::DeducedAbsent),
            }
        };

        // All songs are provided along with the `LevelsRequest` response. A cache miss above means that
        // the GD servers failed to provide the song - there's nothing we can do about it, so we just
        // return a future that resolves to `None` here (making a LevelsRequest would obviously lead to an
        // infinite loop of sorts)
        let refresh = move |_: &PartialLevel<u64, u64>| ok(CacheEntry::DeducedAbsent);

        self.levels(request)?
            .extend_all(lookup, refresh, |p, q| Some(exchange::partial_level_song(p, q)))
    }
}

impl<A, C, Song> ProcessRequest<A, C, LevelsRequest, Vec<PartialLevel<Song, Option<User>>>> for Gdcf<A, C>
where
    A: ApiClient + MakeRequest<LevelsRequest> + MakeRequest<UserRequest>,
    C: Cache + Store<Creator> + Store<NewgroundsSong> + CanCache<LevelsRequest> + CanCache<UserRequest> + Lookup<Creator>,
    Song: PartialEq + Send + Clone + 'static,
    Gdcf<A, C>: ProcessRequest<A, C, LevelsRequest, Vec<PartialLevel<Song, Option<Creator>>>>,
{
    fn process_request(
        &self,
        request: LevelsRequest,
    ) -> Result<GdcfFuture<Vec<PartialLevel<Song, Option<User>>>, <A as ApiClient>::Err, C>, C::Err> {
        let cache = self.cache();
        let gdcf = self.clone();

        let lookup = move |level: &PartialLevel<Song, Option<Creator>>| {
            level
                .creator
                .as_ref()
                .and_then(|creator| creator.account_id)
                .map(|account_id| cache.lookup(account_id))
                .unwrap_or(Ok(CacheEntry::DeducedAbsent))
        };

        let refresh = move |level: &PartialLevel<Song, Option<Creator>>| {
            gdcf.refresh(UserRequest::new(level.creator.as_ref().unwrap().account_id.unwrap()))
                .then(|result| {
                    match result {
                        Err(GdcfError::Api(ref err)) if err.is_no_result() => Ok(CacheEntry::DeducedAbsent),
                        Err(err) => Err(err),
                        Ok(thing) => Ok(thing),
                    }
                })
        };

        self.levels(request)?
            .extend_all(lookup, refresh, |p, q| Some(exchange::partial_level_user(p, q)))
    }
}

impl<A, C> ProcessRequest<A, C, UserSearchRequest, User> for Gdcf<A, C>
where
    A: ApiClient + MakeRequest<UserSearchRequest> + MakeRequest<UserRequest>,
    C: Cache + CanCache<UserSearchRequest> + CanCache<UserRequest> + CacheUserExt + Store<NewgroundsSong> + Store<Creator>,
    Self: ProcessRequest<A, C, UserRequest, User>,
{
    fn process_request(&self, request: UserSearchRequest) -> Result<GdcfFuture<User, <A as ApiClient>::Err, C>, <C as Cache>::Err> {
        let cache = self.cache();
        let clone = self.clone();

        match cache.username_to_account_id(&request.search_string) {
            Some(account_id) => self.user(account_id),
            None => {
                let lookup = move |searched_user: &SearchedUser| {
                    error!("Username to account ID resolution failed although (Searched)User was cached. This is a bug in the cache implementation of `CacheUserExt`!");

                    cache.lookup(searched_user.account_id)
                };
                let refresh = move |searched_user: &SearchedUser| clone.refresh(UserRequest::new(searched_user.account_id));
                let combinator = |_, user| user;

                self.search_user(request)?.extend(lookup, refresh, combinator)
            },
        }
    }
}*/

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
    pub fn level<Song, User>(&self, request: impl Into<LevelRequest>) -> Result<GdcfFuture<Level<Song, User>, A::Err, C>, C::Err>
    where
        Self: ProcessRequestOld<A, C, LevelRequest, Level<Song, User>>,
        Song: PartialEq,
        User: PartialEq,
    {
        self.process_request_old(request.into())
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
    pub fn levels<Song, User>(
        &self,
        request: impl Into<LevelsRequest>,
    ) -> Result<GdcfFuture<Vec<PartialLevel<Song, User>>, A::Err, C>, C::Err>
    where
        Self: ProcessRequestOld<A, C, LevelsRequest, Vec<PartialLevel<Song, User>>>,
        Song: PartialEq,
        User: PartialEq,
    {
        self.process_request_old(request.into())
    }

    /// Generates a stream of pages of levels by incrementing the [`LevelsRequest`]'s `page`
    /// parameter until it hits the first empty page.
    pub fn paginate_levels<Song, User>(
        &self,
        request: impl Into<LevelsRequest>,
    ) -> Result<GdcfStream<A, C, LevelsRequest, Vec<PartialLevel<Song, User>>, Self>, C::Err>
    where
        Self: ProcessRequestOld<A, C, LevelsRequest, Vec<PartialLevel<Song, User>>>,
        Song: PartialEq,
        User: PartialEq,
    {
        self.paginate(request.into())
    }

    /// Processes the given [`UserRequest`]
    pub fn user(&self, request: impl Into<UserRequest>) -> Result<GdcfFuture<User, A::Err, C>, C::Err>
    where
        Self: ProcessRequestOld<A, C, UserRequest, User>,
    {
        self.process_request_old(request.into())
    }

    pub fn search_user<U>(&self, request: impl Into<UserSearchRequest>) -> Result<GdcfFuture<U, A::Err, C>, C::Err>
    where
        Self: ProcessRequestOld<A, C, UserSearchRequest, U>,
    {
        self.process_request_old(request.into())
    }

    pub fn profile_comments(&self, request: impl Into<ProfileCommentsRequest>) -> Result<GdcfFuture<Vec<ProfileComment>, A::Err, C>, C::Err>
    where
        Self: ProcessRequestOld<A, C, ProfileCommentsRequest, Vec<ProfileComment>>,
    {
        self.process_request_old(request.into())
    }

    pub fn paginate_profile_comments(
        &self,
        request: impl Into<ProfileCommentsRequest>,
    ) -> Result<GdcfStream<A, C, ProfileCommentsRequest, Vec<ProfileComment>, Self>, C::Err>
    where
        Self: ProcessRequestOld<A, C, ProfileCommentsRequest, Vec<ProfileComment>>,
    {
        self.paginate(request.into())
    }
}
