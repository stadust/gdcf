#![feature(never_type)]
#![feature(try_from)]
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
extern crate base64;
pub extern crate chrono;
extern crate futures;
#[macro_use]
extern crate gdcf_derive;
extern crate joinery;
#[macro_use]
extern crate lazy_static;
#[macro_use]
extern crate log;
extern crate percent_encoding;
#[cfg(feature = "deser")]
extern crate serde;
#[cfg(feature = "deser")]
#[macro_use]
extern crate serde_derive;
extern crate failure;

use api::{
    request::{LevelRequest, LevelsRequest, PaginatableRequest, Request, UserRequest},
    ApiClient,
};
use cache::{Cache, CachedObject};
use error::{ApiError, CacheError, GdcfError};
use failure::Fail;
use futures::{
    future::{result, Either, FutureResult},
    task, Async, Future, Stream,
};
use model::{user::DELETED, Creator, GDObject, Level, NewgroundsSong, PartialLevel, User};
use std::{
    mem,
    sync::{Arc, Mutex, MutexGuard},
};

#[macro_use]
mod macros;

pub mod api;
pub mod cache;
pub mod convert;
pub mod error;
mod exchange;
pub mod model;

// TODO: for levels, get their creator via the getGJProfile endpoint, then we can give PartialLevel
// a User

pub trait ProcessRequest<A: ApiClient, C: Cache, R: Request, T> {
    fn process_request(&self, request: R) -> GdcfFuture<T, GdcfError<A::Err, C::Err>>;

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

#[derive(Debug)]
pub struct Gdcf<A, C>
where
    A: ApiClient,
    C: Cache + 'static,
{
    client: Arc<A>,
    cache: Arc<Mutex<C>>,
}

impl<A, C> Clone for Gdcf<A, C>
where
    A: ApiClient,
    C: Cache,
{
    fn clone(&self) -> Self {
        Gdcf {
            client: self.client.clone(),
            cache: self.cache.clone(),
        }
    }
}

impl<A, C> ProcessRequest<A, C, UserRequest, User> for Gdcf<A, C>
where
    A: ApiClient,
    C: Cache,
{
    fn process_request(&self, request: UserRequest) -> GdcfFuture<User, GdcfError<A::Err, C::Err>> {
        info!("Processing request {}", request);

        gdcf! {
            self, request, lookup_user, || {
                let cache = self.cache.clone();

                self.client.user(request)
                    .map_err(GdcfError::Api)
                    .and_then(collect_one!(cache, User))
            }
        }
    }
}

impl<A, C> ProcessRequest<A, C, LevelRequest, Level<u64, u64>> for Gdcf<A, C>
where
    A: ApiClient,
    C: Cache,
{
    fn process_request(&self, request: LevelRequest) -> GdcfFuture<Level<u64, u64>, GdcfError<A::Err, C::Err>> {
        info!("Processing request {} with 'u64' as Song type and 'u64' as User type", request);

        gdcf! {
            self, request, lookup_level, || {
                let cache = self.cache.clone();

                self.client.level(request)
                    .map_err(GdcfError::Api)
                    .and_then(collect_one!(cache, Level))
            }
        }
    }
}

impl<A, C> ProcessRequest<A, C, LevelsRequest, Vec<PartialLevel<u64, u64>>> for Gdcf<A, C>
where
    A: ApiClient,
    C: Cache,
{
    fn process_request(&self, request: LevelsRequest) -> GdcfFuture<Vec<PartialLevel<u64, u64>>, GdcfError<A::Err, C::Err>> {
        info!("Processing request {} with 'u64' as Song type and 'u64' as User type", request);

        gdcf! {
            self, request, lookup_partial_levels, || {
                let cache = self.cache.clone();

                self.client.levels(request.clone())
                    .map_err(GdcfError::Api)
                    .and_then(collect_many!(request, cache, store_partial_levels, PartialLevel))
            }
        }
    }
}

impl<A, C, User> ProcessRequest<A, C, LevelRequest, Level<NewgroundsSong, User>> for Gdcf<A, C>
where
    Self: ProcessRequest<A, C, LevelRequest, Level<u64, User>>,
    A: ApiClient,
    C: Cache,
    User: PartialEq + Send + 'static,
{
    fn process_request(&self, request: LevelRequest) -> GdcfFuture<Level<NewgroundsSong, User>, GdcfError<A::Err, C::Err>> {
        info!(
            "Processing request {} with 'NewgroundsSong' as Song type for arbitrary User type",
            request
        );

        // When simply downloading a level, we do not get its song, only the song ID. The song itself is
        // only provided for a LevelsRequest

        let raw: GdcfFuture<Level<u64, User>, _> = self.process_request(request);
        let gdcf = self.clone();

        // TODO: reintroduce debugging statements

        match raw {
            GdcfFuture {
                cached: Some(cached),
                inner: None,
            } =>
            // In this case, we have the level cached and up-to-date
                match cached.inner().base.custom_song {
                    // Level uses a main song, we dont need to do anything apart from changing the generic type
                    None => GdcfFuture::up_to_date(cached.map(|inner| exchange::level_song(inner, None))),

                    // Level uses a custom song.
                    Some(custom_song_id) => {
                        // We cannot do the lookup in the match because then the cache would be locked for the entire match
                        // block which would deadlock because of the `process_request` call in it.
                        let lookup = self.cache().lookup_song(custom_song_id);

                        match lookup {
                            // The custom song is cached, replace the ID with actual song object and change generic type
                            Ok(song) => GdcfFuture::up_to_date(cached.map(|inner| exchange::level_song(inner, Some(song.extract())))),

                            // The custom song isn't cached, make a request that's sure to put it into the cache, then perform the exchange
                            Err(CacheError::CacheMiss) => {
                                let cached = cached.extract();

                                warn!("The level requested was cached, but not its song, performing a request to retrieve it!");

                                GdcfFuture::absent(
                                    self.levels::<u64, u64>(LevelsRequest::default().with_id(cached.base.level_id))
                                        .and_then(move |_| {
                                            let song = gdcf.cache().lookup_song(custom_song_id)?;

                                            Ok(exchange::level_song(cached, Some(song.extract())))
                                        }),
                                )
                            },

                            // Cache lookup failed, create future that resolves to error instantly
                            Err(err) => GdcfFuture::error(err),
                        }
                    },
                },

            GdcfFuture { cached, inner: Some(f) } => {
                // In this case we have it cached, but not up to date, or not cached at all
                let cached = match cached {
                    Some(cached) =>
                    // If we have it cached, we need to update the cached value either with its custom song from the cache, if that exists.
                    // If it doesn't, we will end up creating a future that does not contain any cached object.
                        match cached.inner().base.custom_song {
                            None => Some(cached.map(|inner| exchange::level_song(inner, None))),
                            Some(custom_song_id) =>
                                match self.cache().lookup_song(custom_song_id) {
                                    Ok(song) => Some(cached.map(|inner| exchange::level_song(inner, Some(song.extract())))),

                                    Err(CacheError::CacheMiss) => None,

                                    Err(err) => return GdcfFuture::error(err),
                                },
                        },
                    None => None, // Level itself wasn't cached already
                };

                GdcfFuture::new(
                    cached,
                    Some(f.and_then(move |level| {
                        if let Some(song_id) = level.base.custom_song {
                            // We cannot do the lookup in the match because then the cache would be locked for the entire match
                            // block which would deadlock because of the `process_request` call in it.
                            let lookup = gdcf.cache().lookup_song(song_id);

                            match lookup {
                                // Here we must have this logic inside of the future. If we were to lookup the song_id we got from the
                                // (potentially) cached object, it might be outdated, leaving us with an up-to-date level object that
                                // contains a NewgroundsSong object, which does not represent the song the level uses (because the song was
                                // changed between now and the last time the level was cached)
                                Err(CacheError::CacheMiss) => {
                                    warn!(
                                        "The level the song for the requested level was not cached, performing a request to retrieve it!"
                                    );

                                    Either::A(
                                        gdcf.levels::<u64, u64>(LevelsRequest::default().with_id(level.base.level_id))
                                            .and_then(move |_| {
                                                let song = gdcf.cache().lookup_song(song_id)?;

                                                Ok(exchange::level_song(level, Some(song.extract())))
                                            }),
                                    )
                                },

                                Err(err) => Either::B(result(Err(GdcfError::Cache(err)))),

                                Ok(song) => Either::B(result(Ok(exchange::level_song(level, Some(song.extract()))))),
                            }
                        } else {
                            Either::B(result(Ok(exchange::level_song(level, None))))
                        }
                    })),
                )
            },

            _ => unreachable!(),
        }
    }
}

impl<A, C, User> ProcessRequest<A, C, LevelsRequest, Vec<PartialLevel<NewgroundsSong, User>>> for Gdcf<A, C>
where
    Self: ProcessRequest<A, C, LevelsRequest, Vec<PartialLevel<u64, User>>>,
    A: ApiClient,
    C: Cache,
    User: PartialEq + Send + 'static,
{
    fn process_request(&self, request: LevelsRequest) -> GdcfFuture<Vec<PartialLevel<NewgroundsSong, User>>, GdcfError<A::Err, C::Err>> {
        info!("Processing request {} with 'NewgroundsSong' as Song type", request);

        let GdcfFuture { cached, inner } = self.process_request(request);
        let cache = self.cache.clone();

        let processor = move |levels: Vec<PartialLevel<u64, User>>| {
            let cache = cache.lock().unwrap();
            let mut vec = Vec::new();

            for partial_level in levels {
                let built = match partial_level.custom_song {
                    Some(custom_song_id) =>
                        match cache.lookup_song(custom_song_id) {
                            Ok(song) => exchange::partial_level_song(partial_level, Some(song.extract())),

                            Err(CacheError::CacheMiss) => unreachable!(),

                            Err(err) => return Err(err.into()),
                        },

                    None => exchange::partial_level_song(partial_level, None),
                };

                vec.push(built);
            }

            Ok(vec)
        };

        let cached = match cached {
            Some(cached) =>
                match cached.try_map(processor.clone()) {
                    Ok(cached) => Some(cached),
                    Err(err) => return GdcfFuture::error(err),
                },
            None => None,
        };

        GdcfFuture::new(cached, inner.map(|fut| fut.and_then(processor)))
    }
}

impl<A, C> ProcessRequest<A, C, LevelsRequest, Vec<PartialLevel<u64, Creator>>> for Gdcf<A, C>
where
    A: ApiClient,
    C: Cache,
{
    fn process_request(&self, request: LevelsRequest) -> GdcfFuture<Vec<PartialLevel<u64, Creator>>, GdcfError<A::Err, C::Err>> {
        info!("Processing request {} with 'Creator' as User type", request);

        let GdcfFuture { cached, inner } = self.process_request(request);
        let cache = self.cache.clone();

        let processor = move |levels: Vec<PartialLevel<u64, u64>>| {
            let cache = cache.lock().unwrap();
            let mut vec = Vec::new();

            for partial_level in levels {
                // Note that we do not need to check if the cache value is out-of-date here, because we only
                // request creators that we put into the cache by the very request whose result we're processing
                // here. I THINK it's impossible to have an outdated creator while not having the level request
                // outdated we well.
                vec.push(match cache.lookup_creator(partial_level.creator) {
                    Ok(creator) => exchange::partial_level_user(partial_level, creator.extract()),

                    // For very old levels where the players never registered, the accounts got lost somehow. LevelsRequest containing such
                    // levels don't contain any creator info about those levels. This again implies that the cache miss, which should be
                    // impossible, is such a case.
                    Err(CacheError::CacheMiss) => exchange::partial_level_user(partial_level, DELETED.clone()),

                    Err(err) => return Err(err.into()),
                })
            }

            Ok(vec)
        };

        let cached = match cached {
            Some(cached) =>
                match cached.try_map(processor.clone()) {
                    Ok(cached) => Some(cached),
                    Err(err) => return GdcfFuture::error(err),
                },
            None => None,
        };

        GdcfFuture::new(cached, inner.map(|fut| fut.and_then(processor)))
    }
}

impl<A, C> ProcessRequest<A, C, LevelRequest, Level<u64, Creator>> for Gdcf<A, C>
where
    A: ApiClient,
    C: Cache,
{
    fn process_request(&self, request: LevelRequest) -> GdcfFuture<Level<u64, Creator>, GdcfError<A::Err, C::Err>> {
        // Note that the creator of a level cannot change. We can always use the user ID cached with the
        // level (if existing).
        let raw: GdcfFuture<Level<u64, u64>, _> = self.process_request(request);
        let gdcf = self.clone();

        match raw {
            GdcfFuture {
                cached: Some(cached),
                inner: None,
            } =>
                match self.cache().lookup_creator(cached.inner().base.creator) {
                    Ok(creator) => GdcfFuture::up_to_date(cached.map(|inner| exchange::level_user(inner, creator.extract()))),

                    Err(CacheError::CacheMiss) => {
                        let cached = cached.extract();

                        GdcfFuture::absent(
                            self.levels::<u64, u64>(LevelsRequest::default().with_id(cached.base.level_id))
                                .and_then(move |_| {
                                    let lookup = gdcf.cache().lookup_creator(cached.base.creator);

                                    match lookup {
                                        Ok(creator) => Ok(exchange::level_user(cached, creator.extract())),

                                        Err(CacheError::CacheMiss) => Ok(exchange::level_user(cached, DELETED.clone())),

                                        Err(err) => Err(GdcfError::Cache(err)),
                                    }
                                }),
                        )
                    },

                    Err(err) => GdcfFuture::error(GdcfError::Cache(err)),
                },

            GdcfFuture { cached, inner: Some(f) } => {
                let cached = match cached {
                    Some(cached) =>
                        match self.cache().lookup_creator(cached.inner().base.creator) {
                            Ok(creator) => Some(cached.map(|inner| exchange::level_user(inner, creator.extract()))),

                            Err(CacheError::CacheMiss) => None, /* NOTE: here we cannot decide whether the creator isn't cached, or */
                            // whether his GD account was deleted. We go with the conversative
                            // option and assume it wasn't cached.
                            Err(err) => return GdcfFuture::error(GdcfError::Cache(err)),
                        },

                    None => None,
                };

                GdcfFuture::new(
                    cached,
                    Some(f.and_then(move |level| {
                        let lookup = gdcf.cache().lookup_creator(level.base.creator);

                        match lookup {
                            Ok(creator) => Either::B(result(Ok(exchange::level_user(level, creator.extract())))),

                            Err(CacheError::CacheMiss) =>
                                Either::A(
                                    gdcf.levels::<u64, u64>(LevelsRequest::default().with_id(level.base.level_id))
                                        .and_then(move |_| {
                                            let lookup = gdcf.cache().lookup_creator(level.base.creator);

                                            match lookup {
                                                Ok(creator) => Ok(exchange::level_user(level, creator.extract())),

                                                Err(CacheError::CacheMiss) => Ok(exchange::level_user(level, DELETED.clone())),

                                                Err(err) => Err(GdcfError::Cache(err)),
                                            }
                                        }),
                                ),

                            Err(err) => Either::B(result(Err(GdcfError::Cache(err)))),
                        }
                    })),
                )
            },

            _ => unreachable!(),
        }
    }
}

// TODO: impl ProcessRequest<LevelsRequest, Vec<PartialLevel<u64, User>>> for Gdcf

// TODO: impl ProcessRequest<LevelRequest, Level<u64, User>> for Gdcf

/*impl<A, C> ProcessRequest<A, C, LevelsRequest, Vec<PartialLevel<u64, User>>> for Gdcf<A, C>
where
    A: ApiClient,
    C: Cache,
{
    fn process_request(&self, request: LevelsRequest) -> GdcfFuture<Vec<PartialLevel<u64, User>>, A::Err, C::Err> {
        let raw: GdcfFuture<Vec<PartialLevel<u64, u64>>, _, _> = self.process_request(request);
        let cache = self.cache.clone();

        match raw {
            GdcfFuture {cached: Some(cached), inner: None} => {
                match self.cache().lookup_user(&cached.creator.into()) {
                    Ok(user) => GdcfFuture::up_to_date(build::partial_level_user(user)),

                    Err(CacheError::CacheMiss) => {
                        //GdcfFuture::absent(self.user)
                        unimplemented!()
                    }

                    Err(err) => GdcfFuture::error(err.into())
                }
            }

            _ => unreachable!()
        }
    }
}*/

impl<A, C> Gdcf<A, C>
where
    A: ApiClient,
    C: Cache + 'static,
{
    pub fn new(client: A, cache: C) -> Gdcf<A, C> {
        Gdcf {
            client: Arc::new(client),
            cache: Arc::new(Mutex::new(cache)),
        }
    }

    pub fn cache(&self) -> MutexGuard<C> {
        self.cache.lock().unwrap()
    }

    pub fn client(&self) -> Arc<A> {
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
    pub fn level<Song, User>(&self, request: LevelRequest) -> GdcfFuture<Level<Song, User>, GdcfError<A::Err, C::Err>>
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
    pub fn levels<Song, User>(&self, request: LevelsRequest) -> GdcfFuture<Vec<PartialLevel<Song, User>>, GdcfError<A::Err, C::Err>>
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
    pub fn user(&self, request: UserRequest) -> GdcfFuture<User, GdcfError<A::Err, C::Err>> {
        self.process_request(request)
    }
}

#[allow(missing_debug_implementations)]
pub struct GdcfFuture<T, E: Fail> {
    // invariant: at least one of the fields is not `None`
    pub cached: Option<CachedObject<T>>,
    pub inner: Option<Box<dyn Future<Item = T, Error = E> + Send + 'static>>,
}

impl<T, E: Fail> GdcfFuture<T, E> {
    fn new<F>(cached: Option<CachedObject<T>>, f: Option<F>) -> GdcfFuture<T, E>
    where
        F: Future<Item = T, Error = E> + Send + 'static,
    {
        GdcfFuture {
            cached,
            inner: match f {
                None => None,
                Some(f) => Some(Box::new(f)),
            },
        }
    }

    fn up_to_date(object: CachedObject<T>) -> GdcfFuture<T, E> {
        GdcfFuture {
            cached: Some(object),
            inner: None,
        }
    }

    fn outdated<F>(object: CachedObject<T>, f: F) -> GdcfFuture<T, E>
    where
        F: Future<Item = T, Error = E> + Send + 'static,
    {
        GdcfFuture::new(Some(object), Some(f))
    }

    fn absent<F>(f: F) -> GdcfFuture<T, E>
    where
        F: Future<Item = T, Error = E> + Send + 'static,
    {
        GdcfFuture::new(None, Some(f))
    }

    fn error<Err: Into<E>>(error: Err) -> Self
    where
        T: Send + 'static,
    {
        GdcfFuture::new(None, Some(result(Err(error.into()))))
    }

    pub fn cached(&self) -> &Option<CachedObject<T>> {
        &self.cached
    }
}

impl<T, E: Fail> Future for GdcfFuture<T, E> {
    type Error = E;
    type Item = T;

    fn poll(&mut self) -> Result<Async<T>, E> {
        match self.inner {
            Some(ref mut fut) => fut.poll(),
            None => Ok(Async::Ready(self.cached.take().unwrap().extract())),
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
    current_request: GdcfFuture<T, GdcfError<A::Err, C::Err>>,
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

    fn poll(&mut self) -> Result<Async<Option<T>>, GdcfError<A::Err, C::Err>> {
        match self.current_request.poll() {
            Ok(Async::NotReady) => Ok(Async::NotReady),

            Ok(Async::Ready(result)) => {
                task::current().notify();

                let next = self.next_request.next();
                let cur = mem::replace(&mut self.next_request, next);

                self.current_request = self.source.process_request(cur);

                Ok(Async::Ready(Some(result)))
            },

            Err(GdcfError::NoContent) | Err(GdcfError::Api(ApiError::NoData)) => {
                info!("Stream over request {} terminating due to exhaustion!", self.next_request);

                Ok(Async::Ready(None))
            },

            Err(err) => Err(err),
        }
    }
}
