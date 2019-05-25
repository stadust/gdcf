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

// TODO: it would be nice to be able to differentiate between cache-miss because the data doesn't
// exist and cache-miss because the data simply wasn't requested yet

use crate::{
    api::{
        client::{MakeRequest, Response},
        request::{LevelRequest, LevelsRequest, PaginatableRequest, Request, UserRequest},
        ApiClient,
    },
    cache::{Cache, CacheEntry, CanCache, Lookup, Store},
    error::{ApiError, CacheError, GdcfError},
};
use futures::{
    future::{err, join_all, ok, Either},
    task, Async, Future, Stream,
};
use gdcf_model::{
    level::{Level, PartialLevel},
    song::NewgroundsSong,
    user::{Creator, User},
};
use log::{info, warn};
use std::mem;

#[macro_use]
mod macros;

pub mod api;
pub mod cache;
pub mod error;
mod exchange;

// FIXME: move this somewhere more fitting
#[derive(Debug, Clone, PartialEq)]
pub enum Secondary {
    NewgroundsSong(NewgroundsSong),
    Creator(Creator),
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
        }
    }
}

// TODO: for levels, get their creator via the getGJProfile endpoint, then we can give PartialLevel
// a User

pub trait ProcessRequest<A: ApiClient, C: Cache, R: Request, T> {
    fn process_request(&self, request: R) -> GdcfFuture<T, A::Err, C>;

    fn paginate(&self, request: R) -> GdcfStream<A, C, R, T, Self>
    where
        R: PaginatableRequest,
        Self: Sized + Clone,
    {
        let next = request.next();
        let current = self.process_request(request);

        GdcfStream {
            next_request: next,
            current_request: current,
            source: self.clone(),
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
    fn refresh<R>(&self, request: R) -> impl Future<Item = CacheEntry<R::Result, C>, Error = GdcfError<A::Err, C::Err>>
    where
        R: Request,
        A: MakeRequest<R>,
        C: CanCache<R>,
    {
        let mut cache = self.cache();
        let key = request.key();

        self.client().make(request).map_err(GdcfError::Api).and_then(move |response| {
            match response {
                Response::Exact(what_we_want) =>
                    cache
                        .store(&what_we_want, key)
                        .map(move |entry_info| {
                            CacheEntry {
                                object: what_we_want,
                                metadata: entry_info,
                            }
                        })
                        .map_err(GdcfError::Cache),
                Response::More(what_we_want, excess) => {
                    for object in &excess {
                        match object {
                            Secondary::NewgroundsSong(song) => cache.store(song, song.song_id),
                            Secondary::Creator(creator) => cache.store(creator, creator.user_id),
                        }
                        .map_err(GdcfError::Cache)?;
                    }

                    cache
                        .store(&what_we_want, key)
                        .map(move |entry_info| {
                            CacheEntry {
                                object: what_we_want,
                                metadata: entry_info,
                            }
                        })
                        .map_err(GdcfError::Cache)
                },
            }
        })
    }

    fn process<R>(
        &self, request: R,
    ) -> Result<
        EitherOrBoth<CacheEntry<R::Result, C>, impl Future<Item = CacheEntry<R::Result, C>, Error = GdcfError<A::Err, C::Err>>>,
        C::Err,
    >
    where
        R: Request,
        A: MakeRequest<R>,
        C: CanCache<R>,
    {
        info!("Processing request {}", request);

        let cached = match self.cache.lookup_request(&request) {
            Ok(entry) =>
                if entry.is_expired() {
                    info!("Cache entry for request {} is expired!", request);

                    Some(entry)
                } else {
                    info!("Cached entry for request {} is up-to-date!", request);

                    return Ok(EitherOrBoth::A(entry))
                },

            Err(ref error) if error.is_cache_miss() => {
                info!("No cache entry for request {}", request);

                None
            },

            Err(error) => return Err(error),
        };

        let future = self.refresh(request);

        Ok(match cached {
            Some(value) => EitherOrBoth::Both(value, future),
            None => EitherOrBoth::B(future),
        })
    }
}

impl<A, R, C> ProcessRequest<A, C, R, R::Result> for Gdcf<A, C>
where
    R: Request + Send + Sync + 'static,
    A: ApiClient + MakeRequest<R>,
    C: Cache + Store<Creator> + Store<NewgroundsSong> + CanCache<R>,
{
    fn process_request(&self, request: R) -> GdcfFuture<R::Result, A::Err, C> {
        match self.process(request) {
            Ok(EitherOrBoth::A(entry)) => GdcfFuture::UpToDate(entry),
            Ok(EitherOrBoth::B(future)) => GdcfFuture::Uncached(Box::new(future)),
            Ok(EitherOrBoth::Both(entry, future)) => GdcfFuture::Outdated(entry, Box::new(future)),
            Err(err) => GdcfFuture::CacheError(err),
        }
    }
}

impl<A, C> ProcessRequest<A, C, LevelRequest, Level<NewgroundsSong, u64>> for Gdcf<A, C>
where
    A: ApiClient + MakeRequest<LevelRequest> + MakeRequest<LevelsRequest>,
    C: Cache + Store<Creator> + Store<NewgroundsSong> + CanCache<LevelRequest> + CanCache<LevelsRequest> + Lookup<NewgroundsSong>,
{
    fn process_request(&self, request: LevelRequest) -> GdcfFuture<Level<NewgroundsSong, u64>, <A as ApiClient>::Err, C> {
        let (cache1, cache2) = (self.cache(), self.cache());
        let gdcf = self.clone();

        let lookup = move |level: &Level<u64, u64>| {
            match level.base.custom_song {
                Some(song_id) => cache1.lookup(song_id).map(Some),
                None => Ok(None),
            }
        };

        let refresh = move |level: &Level<u64, u64>| {
            let song_id = level.base.custom_song.unwrap();

            gdcf.refresh(LevelsRequest::default().with_id(level.base.level_id))
                .and_then(move |_| {
                    match cache2.lookup(song_id) {
                        Err(ref err) if err.is_cache_miss() => Ok(None),
                        Err(err) => Err(GdcfError::Cache(err)),
                        Ok(obj) => Ok(Some(obj)),
                    }
                })
        };

        self.level(request).chain(lookup, refresh, exchange::level_song)
    }
}

impl<A, C, Song> ProcessRequest<A, C, LevelRequest, Level<Song, Option<Creator>>> for Gdcf<A, C>
where
    A: ApiClient + MakeRequest<LevelRequest> + MakeRequest<LevelsRequest>,
    C: Cache + Store<Creator> + Store<NewgroundsSong> + CanCache<LevelRequest> + CanCache<LevelsRequest> + Lookup<Creator>,
    Song: PartialEq + Send + Clone + 'static,
    Gdcf<A, C>: ProcessRequest<A, C, LevelRequest, Level<Song, u64>>,
{
    fn process_request(&self, request: LevelRequest) -> GdcfFuture<Level<Song, Option<Creator>>, <A as ApiClient>::Err, C> {
        let cache = self.cache();
        let cache2 = self.cache();
        let gdcf = self.clone();

        let lookup = move |level: &Level<Song, u64>| cache.lookup(level.base.creator).map(Some);
        let refresh = move |level: &Level<Song, u64>| {
            let user_id = level.base.creator;

            gdcf.refresh(LevelsRequest::default().with_id(level.base.level_id))
                .and_then(move |_| {
                    match cache2.lookup(user_id) {
                        Err(ref err) if err.is_cache_miss() => Ok(None),
                        Err(err) => Err(GdcfError::Cache(err)),
                        Ok(obj) => Ok(Some(obj)),
                    }
                })
        };

        self.level(request).chain(lookup, refresh, exchange::level_user)
    }
}

impl<A, C, Song> ProcessRequest<A, C, LevelRequest, Level<Song, Option<User>>> for Gdcf<A, C>
where
    A: ApiClient + MakeRequest<LevelRequest> + MakeRequest<UserRequest>,
    C: Cache + Store<Creator> + Store<NewgroundsSong> + CanCache<LevelRequest> + CanCache<UserRequest>,
    Song: PartialEq + Send + Clone + 'static,
    Gdcf<A, C>: ProcessRequest<A, C, LevelRequest, Level<Song, Option<Creator>>>,
{
    fn process_request(&self, request: LevelRequest) -> GdcfFuture<Level<Song, Option<User>>, <A as ApiClient>::Err, C> {
        let cache = self.cache();
        let gdcf = self.clone();

        let lookup = move |level: &Level<Song, Option<Creator>>| {
            level
                .base
                .creator
                .as_ref()
                .and_then(|creator| creator.account_id)
                .map(|account_id| cache.lookup(account_id))
                .transpose()
        };

        let refresh = move |level: &Level<Song, Option<Creator>>| {
            gdcf.refresh(UserRequest::new(level.base.creator.as_ref().unwrap().account_id.unwrap()))
                .then(|result| {
                    match result {
                        Err(GdcfError::Api(ref err)) if err.is_no_result() => Ok(None),
                        Err(err) => Err(err),
                        Ok(thing) => Ok(Some(thing)),
                    }
                })
        };

        self.level(request).chain(lookup, refresh, exchange::level_user)
    }
}

impl<A, C, Song> ProcessRequest<A, C, LevelsRequest, Vec<PartialLevel<Song, Option<Creator>>>> for Gdcf<A, C>
where
    A: ApiClient + MakeRequest<LevelsRequest>,
    C: Cache + Store<Creator> + Store<NewgroundsSong> + CanCache<LevelsRequest> + Lookup<Creator>,
    Song: PartialEq + Send + Clone + 'static,
    Gdcf<A, C>: ProcessRequest<A, C, LevelsRequest, Vec<PartialLevel<Song, u64>>>,
{
    fn process_request(&self, request: LevelsRequest) -> GdcfFuture<Vec<PartialLevel<Song, Option<Creator>>>, <A as ApiClient>::Err, C> {
        let cache = self.cache();

        let lookup = move |level: &PartialLevel<Song, u64>| cache.lookup(level.creator).map(Some);

        // All creators are provided along with the `LevelsRequest` response. A cache miss above means that
        // the GD servers failed to provide the creator - there's nothing we can do about it, so we just
        // return a future that resolves to `None` here (making a LevelsRequest would obviously lead to an
        // infinite loop of sorts)
        let refresh = move |level: &PartialLevel<Song, u64>| ok(None);

        self.levels(request).multi_chain(lookup, refresh, exchange::partial_level_user)
    }
}

impl<A, C> ProcessRequest<A, C, LevelsRequest, Vec<PartialLevel<NewgroundsSong, u64>>> for Gdcf<A, C>
where
    A: ApiClient + MakeRequest<LevelsRequest>,
    C: Cache + Store<Creator> + Store<NewgroundsSong> + CanCache<LevelsRequest> + Lookup<NewgroundsSong>,
    Gdcf<A, C>: ProcessRequest<A, C, LevelsRequest, Vec<PartialLevel<u64, u64>>>,
{
    fn process_request(&self, request: LevelsRequest) -> GdcfFuture<Vec<PartialLevel<NewgroundsSong, u64>>, <A as ApiClient>::Err, C> {
        let cache = self.cache();

        let lookup = move |level: &PartialLevel<u64, u64>| {
            match level.custom_song {
                Some(song_id) => cache.lookup(song_id).map(Some),
                None => Ok(None),
            }
        };

        // All songs are provided along with the `LevelsRequest` response. A cache miss above means that
        // the GD servers failed to provide the song - there's nothing we can do about it, so we just
        // return a future that resolves to `None` here (making a LevelsRequest would obviously lead to an
        // infinite loop of sorts)
        let refresh = move |level: &PartialLevel<u64, u64>| ok(None);

        self.levels(request).multi_chain(lookup, refresh, exchange::partial_level_song)
    }
}

impl<A, C, Song> ProcessRequest<A, C, LevelsRequest, Vec<PartialLevel<Song, Option<User>>>> for Gdcf<A, C>
where
    A: ApiClient + MakeRequest<LevelsRequest> + MakeRequest<UserRequest>,
    C: Cache + Store<Creator> + Store<NewgroundsSong> + CanCache<LevelsRequest> + CanCache<UserRequest> + Lookup<Creator>,
    Song: PartialEq + Send + Clone + 'static,
    Gdcf<A, C>: ProcessRequest<A, C, LevelsRequest, Vec<PartialLevel<Song, Option<Creator>>>>,
{
    fn process_request(&self, request: LevelsRequest) -> GdcfFuture<Vec<PartialLevel<Song, Option<User>>>, <A as ApiClient>::Err, C> {
        let cache = self.cache();
        let gdcf = self.clone();

        let lookup = move |level: &PartialLevel<Song, Option<Creator>>| {
            level
                .creator
                .as_ref()
                .and_then(|creator| creator.account_id)
                .map(|account_id| cache.lookup(account_id))
                .transpose()
        };

        let refresh = move |level: &PartialLevel<Song, Option<Creator>>| {
            gdcf.refresh(UserRequest::new(level.creator.as_ref().unwrap().account_id.unwrap()))
                .then(|result| {
                    match result {
                        Err(GdcfError::Api(ref err)) if err.is_no_result() => Ok(None),
                        Err(err) => Err(err),
                        Ok(thing) => Ok(Some(thing)),
                    }
                })
        };

        self.levels(request).multi_chain(lookup, refresh, exchange::partial_level_user)
    }
}

impl<A, C> Gdcf<A, C>
where
    A: ApiClient,
    C: Cache + Store<NewgroundsSong> + Store<Creator>,
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
    pub fn level<Song, User>(&self, request: LevelRequest) -> GdcfFuture<Level<Song, User>, A::Err, C>
    where
        Self: ProcessRequest<A, C, LevelRequest, Level<Song, User>>,
        Song: PartialEq,
        User: PartialEq,
    {
        self.process_request(request)
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
    /// profile (Not Yet Implemented)
    ///
    /// `Song` can currently be one of the following:
    /// + [`u64`] - The custom song is provided only as its newgrounds ID. Causes no additional
    /// requests
    /// + [`NewgroundsSong`] - Causes no additional requests.
    pub fn levels<Song, User>(&self, request: LevelsRequest) -> GdcfFuture<Vec<PartialLevel<Song, User>>, A::Err, C>
    where
        Self: ProcessRequest<A, C, LevelsRequest, Vec<PartialLevel<Song, User>>>,
        Song: PartialEq,
        User: PartialEq,
    {
        self.process_request(request)
    }

    /// Generates a stream of pages of levels by incrementing the [`LevelsRequest`]'s `page`
    /// parameter until it hits the first empty page.
    pub fn paginate_levels<Song, User>(
        &self, request: LevelsRequest,
    ) -> impl Stream<Item = Vec<PartialLevel<Song, User>>, Error = GdcfError<A::Err, C::Err>>
    where
        Self: ProcessRequest<A, C, LevelsRequest, Vec<PartialLevel<Song, User>>>,
        Song: PartialEq,
        User: PartialEq,
    {
        self.paginate(request)
    }

    /// Processes the given [`UserRequest`]
    pub fn user(&self, request: UserRequest) -> GdcfFuture<User, A::Err, C>
    where
        Self: ProcessRequest<A, C, UserRequest, User>,
    {
        self.process_request(request)
    }
}

#[allow(missing_debug_implementations)]
pub enum GdcfFuture<T, A: ApiError, C: Cache> {
    Empty,
    CacheError(C::Err),
    Uncached(Box<dyn Future<Item = CacheEntry<T, C>, Error = GdcfError<A, C::Err>> + Send + 'static>),
    Outdated(
        CacheEntry<T, C>,
        Box<dyn Future<Item = CacheEntry<T, C>, Error = GdcfError<A, C::Err>> + Send + 'static>,
    ),
    UpToDate(CacheEntry<T, C>),
}

impl<T, A: ApiError, C: Cache> GdcfFuture<T, A, C> {
    pub fn is_err(&self) -> bool {
        match self {
            GdcfFuture::Empty | GdcfFuture::CacheError(_) => true,
            _ => false,
        }
    }

    pub fn cached(&self) -> Option<&CacheEntry<T, C>> {
        match self {
            GdcfFuture::Outdated(ref entry, _) | GdcfFuture::UpToDate(ref entry) => Some(entry),
            _ => None,
        }
    }

    pub fn inner_future(&self) -> Option<&impl Future<Item = CacheEntry<T, C>, Error = GdcfError<A, C::Err>>> {
        match self {
            GdcfFuture::Uncached(ref fut) | GdcfFuture::Outdated(_, ref fut) => Some(fut),
            _ => None,
        }
    }

    pub fn deconstruct(
        self,
    ) -> Result<
        (
            Option<impl Future<Item = CacheEntry<T, C>, Error = GdcfError<A, C::Err>>>,
            Option<CacheEntry<T, C>>,
        ),
        C::Err,
    > {
        match self {
            GdcfFuture::Empty => Ok((None, None)),
            GdcfFuture::CacheError(err) => Err(err),
            GdcfFuture::Uncached(fut) => Ok((Some(fut), None)),
            GdcfFuture::Outdated(entry, fut) => Ok((Some(fut), Some(entry))),
            GdcfFuture::UpToDate(entry) => Ok((None, Some(entry))),
        }
    }

    fn chain<I, U, Look, Req, Comb, Fut>(self, lookup: Look, request: Req, combinator: Comb) -> GdcfFuture<U, A, C>
    where
        T: Clone + Send + 'static,
        U: Send + 'static,
        I: Clone + Send + 'static,
        Look: FnOnce(&T) -> Result<Option<CacheEntry<I, C>>, C::Err> + Send + 'static,
        Req: FnOnce(&T) -> Fut + Send + 'static,
        Comb: Fn(T, Option<I>) -> U + Send + Sync + 'static,
        Fut: Future<Item = Option<CacheEntry<I, C>>, Error = GdcfError<A, C::Err>> + Send + 'static,
    {
        let combine = move |entry: CacheEntry<T, C>, other: Option<CacheEntry<I, C>>| {
            CacheEntry {
                object: combinator(entry.object, other.map(|inner| inner.object)),
                metadata: entry.metadata,
            }
        };

        match self {
            GdcfFuture::Empty => GdcfFuture::Empty,
            GdcfFuture::CacheError(err) => GdcfFuture::CacheError(err),
            GdcfFuture::UpToDate(unmapped) =>
                match lookup(unmapped.inner()) {
                    Ok(None) => GdcfFuture::UpToDate(combine(unmapped, None)),

                    Ok(Some(add_on)) =>
                        if add_on.is_expired() {
                            GdcfFuture::Outdated(
                                combine(unmapped.clone(), Some(add_on)),
                                Box::new(request(&unmapped.object).map(move |intermediate| combine(unmapped, intermediate))),
                            )
                        } else {
                            GdcfFuture::UpToDate(combine(unmapped, Some(add_on)))
                        },

                    Err(ref err) if err.is_cache_miss() =>
                        GdcfFuture::Uncached(Box::new(
                            request(&unmapped.object).map(move |intermediate| combine(unmapped, intermediate)),
                        )),

                    Err(err) => GdcfFuture::CacheError(err),
                },
            GdcfFuture::Uncached(future) =>
                GdcfFuture::Uncached(Box::new(future.and_then(move |unmapped| {
                    match lookup(unmapped.inner()) {
                        Ok(None) => Either::A(ok(combine(unmapped, None))),

                        Ok(add_on) => {
                            // TODO: handling of result.is_expired()

                            Either::A(ok(combine(unmapped, add_on)))
                        },

                        Err(ref err) if err.is_cache_miss() =>
                            Either::B(request(unmapped.inner()).map(move |intermediate| combine(unmapped, intermediate))),

                        Err(error) => Either::A(err(GdcfError::Cache(error))),
                    }
                }))),
            GdcfFuture::Outdated(unmapped, future) =>
                match lookup(unmapped.inner()) {
                    Ok(add_on) =>
                        if unmapped.is_expired() {
                            let request_future = request(unmapped.inner());

                            GdcfFuture::Outdated(
                                combine(unmapped, add_on),
                                Box::new(
                                    future
                                        .and_then(move |unmapped| request_future.map(move |intermediate| combine(unmapped, intermediate))),
                                ),
                            )
                        } else {
                            GdcfFuture::Outdated(
                                combine(unmapped, add_on.clone()),
                                Box::new(future.map(move |unmapped| combine(unmapped, add_on))),
                            )
                        },

                    Err(ref err) if err.is_cache_miss() =>
                        GdcfFuture::Uncached(Box::new(
                            request(unmapped.inner()).map(move |intermediate| combine(unmapped, intermediate)),
                        )),

                    Err(err) => GdcfFuture::CacheError(err),
                },
        }
    }
}

impl<T, A: ApiError, C: Cache> GdcfFuture<Vec<T>, A, C> {
    fn multi_chain<I, U, Look, Req, Comb, Fut>(self, lookup: Look, request: Req, combinator: Comb) -> GdcfFuture<Vec<U>, A, C>
    where
        T: Clone + Send + 'static,
        U: Clone + Send + 'static,
        I: Clone + Send + 'static,
        Look: Fn(&T) -> Result<Option<CacheEntry<I, C>>, C::Err> + Send + 'static,
        Req: Fn(&T) -> Fut + Send + 'static,
        Comb: Copy + Fn(T, Option<I>) -> U + Send + Sync + 'static,
        Fut: Future<Item = Option<CacheEntry<I, C>>, Error = GdcfError<A, C::Err>> + Send + 'static,
    {
        let combine = move |entry: T, other: Option<CacheEntry<I, C>>| combinator(entry, other.map(|inner| inner.object));

        match self {
            GdcfFuture::Empty => GdcfFuture::Empty,
            GdcfFuture::CacheError(err) => GdcfFuture::CacheError(err),
            GdcfFuture::UpToDate(CacheEntry {
                object: unmapped,
                metadata,
            }) => {
                let mut objects: Vec<U> = Vec::new();
                let mut futures = Vec::new();

                let mut outdated = false;

                for object in unmapped {
                    match match lookup(&object) {
                        Ok(None) => (None, Some(combine(object, None))),

                        Ok(Some(add_on)) =>
                            if !add_on.is_expired() {
                                (None, Some(combine(object, Some(add_on))))
                            } else {
                                (Some(object.clone()), Some(combine(object, Some(add_on))))
                            },

                        Err(ref err) if err.is_cache_miss() => (Some(object), None),

                        Err(err) => return GdcfFuture::CacheError(err),
                    } {
                        (None, Some(combined)) => {
                            // Ehh, we can probably prevent some clones here by only constructing the futures later or sth
                            futures.push(Either::B(ok(combined.clone())));
                            objects.push(combined);
                        },
                        (Some(object), combined) => {
                            outdated = true;

                            if let Some(combined) = combined {
                                objects.push(combined);
                            }
                            futures.push(Either::A(request(&object).map(move |intermediate| combine(object, intermediate))))
                        },
                        _ => unreachable!(),
                    }
                }

                let uncached = objects.len() != futures.len();
                let mdc = metadata.clone();
                let joined_future = join_all(futures).map(move |results| {
                    CacheEntry {
                        object: results,
                        metadata: mdc,
                    }
                });

                match (outdated, uncached) {
                    (false, false) => GdcfFuture::UpToDate(CacheEntry { object: objects, metadata }),
                    (true, false) => GdcfFuture::Outdated(CacheEntry { object: objects, metadata }, Box::new(joined_future)),
                    (_, true) => GdcfFuture::Uncached(Box::new(joined_future)),
                }
            },
            GdcfFuture::Uncached(future) =>
                GdcfFuture::Uncached(Box::new(future.and_then(
                    move |CacheEntry {
                              object: unmapped,
                              metadata,
                          }| {
                        join_all(unmapped.into_iter().map(move |object| {
                            match lookup(&object) {
                                Ok(add_on) => {
                                    // TODO: handling of result.is_expired()

                                    Either::A(ok(combine(object, add_on)))
                                },

                                Err(ref err) if err.is_cache_miss() =>
                                    Either::B(request(&object).map(move |intermediate| combine(object, intermediate))),

                                Err(error) => Either::A(err(GdcfError::Cache(error))),
                            }
                        }))
                        .map(move |result| CacheEntry { object: result, metadata })
                    },
                ))),
            GdcfFuture::Outdated(
                CacheEntry {
                    object: unmapped,
                    metadata,
                },
                future,
            ) => {
                let mut objects = Vec::new();

                let mut uncached = false;

                for object in unmapped {
                    match lookup(&object) {
                        Ok(add_on) => {
                            objects.push(combine(object.clone(), add_on));
                        },

                        Err(ref err) if err.is_cache_miss() => {
                            uncached = true;

                            break
                        },

                        Err(err) => return GdcfFuture::CacheError(err),
                    }
                }

                let future = future.and_then(
                    move |CacheEntry {
                              object: unmapped,
                              metadata,
                          }| {
                        join_all(unmapped.into_iter().map(move |object| {
                            match lookup(&object) {
                                Ok(add_on) => {
                                    // TODO: handling of result.is_expired()

                                    Either::A(ok(combine(object, add_on)))
                                },

                                Err(ref err) if err.is_cache_miss() =>
                                    Either::B(request(&object).map(move |intermediate| combine(object, intermediate))),

                                Err(error) => Either::A(err(GdcfError::Cache(error))),
                            }
                        }))
                        .map(move |result| CacheEntry { object: result, metadata })
                    },
                );

                if uncached {
                    GdcfFuture::Uncached(Box::new(future))
                } else {
                    GdcfFuture::Outdated(CacheEntry { object: objects, metadata }, Box::new(future))
                }
            },
        }
    }
}

impl<T, A: ApiError, C: Cache> Future for GdcfFuture<T, A, C> {
    type Error = GdcfError<A, C::Err>;
    type Item = T;

    fn poll(&mut self) -> Result<Async<T>, Self::Error> {
        match self {
            GdcfFuture::Uncached(future) => future.poll().map(|a| a.map(|i| i.into_inner())),
            GdcfFuture::Outdated(_, future) => future.poll().map(|a| a.map(|i| i.into_inner())),
            GdcfFuture::Empty => panic!("Cannot poll resolved future"),
            fut =>
                match std::mem::replace(fut, GdcfFuture::Empty) {
                    GdcfFuture::CacheError(error) => Err(GdcfError::Cache(error)),
                    GdcfFuture::UpToDate(inner) => Ok(Async::Ready(inner.into_inner())),
                    _ => unreachable!(),
                },
        }
    }
}

#[allow(missing_debug_implementations)]
pub struct GdcfStream<A, C, R, T, M>
where
    R: PaginatableRequest,
    M: ProcessRequest<A, C, R, T>,
    A: ApiClient,
    C: Cache,
{
    next_request: R,
    current_request: GdcfFuture<T, A::Err, C>,
    source: M,
}

impl<A, C, R, T, M> Stream for GdcfStream<A, C, R, T, M>
where
    R: PaginatableRequest,
    M: ProcessRequest<A, C, R, T>,
    A: ApiClient,
    C: Cache,
{
    type Error = GdcfError<A::Err, C::Err>;
    type Item = T;

    fn poll(&mut self) -> Result<Async<Option<T>>, Self::Error> {
        match self.current_request.poll() {
            Ok(Async::NotReady) => Ok(Async::NotReady),

            Ok(Async::Ready(result)) => {
                task::current().notify();

                let next = self.next_request.next();
                let cur = mem::replace(&mut self.next_request, next);

                self.current_request = self.source.process_request(cur);

                Ok(Async::Ready(Some(result)))
            },

            Err(GdcfError::Api(ref err)) if err.is_no_result() => {
                info!("Stream over request {} terminating due to exhaustion!", self.next_request);

                Ok(Async::Ready(None))
            },

            Err(err) => Err(err),
        }
    }
}
