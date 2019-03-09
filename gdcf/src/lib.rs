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

// TODO: it would be nice to be able to differentiate between cache-miss because the data doesn't
// exist and cache-miss because the data simply wasn't requested yet

use crate::{
    api::{
        client::{MakeRequest, Response},
        request::{LevelRequest, LevelsRequest, PaginatableRequest, Request, UserRequest},
        ApiClient,
    },
    cache::{Cache, CachedObject, CanCache},
    error::GdcfError,
};
use futures::{
    future::{err, ok, Either},
    task, Async, Future, Stream,
};
use gdcf_model::{
    level::{Level, PartialLevel},
    song::{NewgroundsSong, SERVER_SIDED_DATA_INCONSISTENCY_ERROR},
    user::{Creator, User, DELETED},
};
use log::{info, warn};
use std::mem;

#[macro_use]
mod macros;

pub mod api;
pub mod cache;
//pub mod convert;
pub mod error;
mod exchange;
//pub mod model;

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

use crate::error::{ApiError, CacheError};

pub trait ProcessRequest<A: ApiClient, C: Cache, R: Request, T> {
    fn process_request(&self, request: R) -> GdcfFuture<T, A::Err, C::Err>;

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

impl<A, R, C> ProcessRequest<A, C, R, R::Result> for Gdcf<A, C>
where
    R: Request + Send + Sync + 'static,
    A: ApiClient + MakeRequest<R>,
    C: Cache + CanCache<R>,
{
    fn process_request(&self, request: R) -> GdcfFuture<R::Result, A::Err, C::Err> {
        info!("Processing request {}", request);

        let cached = match self.cache.lookup(&request) {
            Ok(cached) =>
                if self.cache.is_expired(&cached) {
                    info!("Cache entry for request {} is expired!", request);

                    Some(cached)
                } else {
                    info!("Cached entry for request {} is up-to-date!", request);

                    return GdcfFuture::up_to_date(cached)
                },

            Err(ref error) if error.is_cache_miss() => {
                info!("No cache entry for request {}", request);

                None
            },

            Err(error) => return GdcfFuture::cache_error(error),
        };

        let request_hash = self.cache.hash(&request);

        let mut cache = self.cache();

        let future = self.client().make(request).map_err(GdcfError::Api).and_then(move |response| {
            match response {
                Response::Exact(what_we_want) =>
                    cache
                        .store(&what_we_want, request_hash)
                        .map(move |_| what_we_want)
                        .map_err(GdcfError::Cache),
                Response::More(what_we_want, excess) => {
                    for object in excess {
                        cache.store_secondary(&object).map_err(GdcfError::Cache)?;
                    }

                    cache
                        .store(&what_we_want, request_hash)
                        .map(move |_| what_we_want)
                        .map_err(GdcfError::Cache)
                },
            }
        });

        match cached {
            Some(value) => GdcfFuture::outdated(value, future),
            None => GdcfFuture::absent(future),
        }
    }
}

#[derive(Debug)]
pub struct Gdcf<A, C>
where
    A: ApiClient,
    C: Cache,
{
    client: A,
    cache: C,
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

impl<A, C, User> ProcessRequest<A, C, LevelRequest, Level<NewgroundsSong, User>> for Gdcf<A, C>
where
    Self: ProcessRequest<A, C, LevelRequest, Level<u64, User>>,
    A: ApiClient + MakeRequest<LevelRequest> + MakeRequest<LevelsRequest>,
    C: Cache + CanCache<LevelRequest> + CanCache<LevelsRequest>,
    User: PartialEq + Send + 'static,
{
    fn process_request(&self, request: LevelRequest) -> GdcfFuture<Level<NewgroundsSong, User>, A::Err, C::Err> {
        info!(
            "Processing request {} with 'NewgroundsSong' as Song type for arbitrary User type",
            request
        );

        // When simply downloading a level, we do not get its song, only the song ID. The song itself is
        // only provided for a LevelsRequest

        let raw = self.level::<u64, User>(request);
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
                            Err(ref err) if err.is_cache_miss() => {
                                let cached = cached.extract();

                                warn!("The level requested was cached, but not its song, performing a request to retrieve it!");

                                GdcfFuture::absent(
                                    self.levels::<u64, u64>(LevelsRequest::default().with_id(cached.base.level_id))
                                        .and_then(move |_| {
                                            let song = gdcf.cache().lookup_song(custom_song_id).map_err(GdcfError::Cache)?;

                                            Ok(exchange::level_song(cached, Some(song.extract())))
                                        }),
                                )
                            },

                            // Cache lookup failed, create future that resolves to error instantly
                            Err(err) => GdcfFuture::cache_error(err),
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

                                    Err(ref err) if err.is_cache_miss() => None,

                                    Err(err) => return GdcfFuture::cache_error(err),
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
                                Err(ref err) if err.is_cache_miss() => {
                                    warn!(
                                        "The level the song for the requested level was not cached, performing a request to retrieve it!"
                                    );

                                    Either::A(
                                        gdcf.levels::<u64, u64>(LevelsRequest::default().with_id(level.base.level_id))
                                            .and_then(move |_| {
                                                let song = gdcf.cache().lookup_song(song_id).map_err(GdcfError::Cache)?;

                                                Ok(exchange::level_song(level, Some(song.extract())))
                                            }),
                                    )
                                },

                                Err(error) => Either::B(err(GdcfError::Cache(error))),

                                Ok(song) => Either::B(ok(exchange::level_song(level, Some(song.extract())))),
                            }
                        } else {
                            Either::B(ok(exchange::level_song(level, None)))
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
    A: ApiClient + MakeRequest<LevelsRequest>,
    C: Cache + CanCache<LevelsRequest>,
    User: PartialEq + Send + 'static,
{
    fn process_request(&self, request: LevelsRequest) -> GdcfFuture<Vec<PartialLevel<NewgroundsSong, User>>, A::Err, C::Err> {
        info!("Processing request {} with 'NewgroundsSong' as Song type", request);

        let GdcfFuture { cached, inner } = self.process_request(request);
        let cache = self.cache.clone();

        let processor = move |levels: Vec<PartialLevel<u64, User>>| {
            let mut vec = Vec::new();

            for partial_level in levels {
                let built = match partial_level.custom_song {
                    Some(custom_song_id) =>
                        match cache.lookup_song(custom_song_id) {
                            Ok(song) => exchange::partial_level_song(partial_level, Some(song.extract())),

                            Err(err) =>
                                if err.is_cache_miss() {
                                    exchange::partial_level_song(partial_level, Some(SERVER_SIDED_DATA_INCONSISTENCY_ERROR()))
                                } else {
                                    return Err(GdcfError::Cache(err))
                                },
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
    A: ApiClient + MakeRequest<LevelsRequest>,
    C: Cache + CanCache<LevelsRequest>,
{
    fn process_request(&self, request: LevelsRequest) -> GdcfFuture<Vec<PartialLevel<u64, Creator>>, A::Err, C::Err> {
        info!("Processing request {} with 'Creator' as User type", request);

        let GdcfFuture { cached, inner } = self.process_request(request);
        let cache = self.cache.clone();

        let processor = move |levels: Vec<PartialLevel<u64, u64>>| {
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
                    Err(err) =>
                        if err.is_cache_miss() {
                            exchange::partial_level_user(partial_level, DELETED())
                        } else {
                            return Err(GdcfError::Cache(err))
                        },
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
    A: ApiClient + MakeRequest<LevelRequest> + MakeRequest<LevelsRequest>,
    C: Cache + CanCache<LevelRequest> + CanCache<LevelsRequest>,
{
    fn process_request(&self, request: LevelRequest) -> GdcfFuture<Level<u64, Creator>, A::Err, C::Err> {
        info!("Processing request {} with 'Creator' as User type", request);

        // Note that the creator of a level cannot change. We can always use the user ID cached with the
        // level (if existing).
        //let raw: GdcfFuture<Level<u64, u64>, _, _> = self.process_request(request);
        let raw = self.level::<u64, u64>(request);
        let gdcf = self.clone();

        match raw {
            GdcfFuture {
                cached: Some(cached),
                inner: None,
            } => {
                let lookup = self.cache().lookup_creator(cached.inner().base.creator);

                match lookup {
                    Ok(creator) => {
                        info!("Level {} up-to-date, returning cached version!", cached.inner());

                        GdcfFuture::up_to_date(cached.map(|inner| exchange::level_user(inner, creator.extract())))
                    },

                    Err(ref err) if err.is_cache_miss() => {
                        warn!(
                            "Level {} was up-to-date, but creator is missing from cache. Constructing LevelsRequest to retrieve creator",
                            cached.inner()
                        );

                        let cached = cached.extract();

                        GdcfFuture::absent(
                            self.levels::<u64, u64>(LevelsRequest::default().with_id(cached.base.level_id))
                                .and_then(move |_| {
                                    let lookup = gdcf.cache().lookup_creator(cached.base.creator);

                                    match lookup {
                                        Ok(creator) => Ok(exchange::level_user(cached, creator.extract())),

                                        Err(ref err) if err.is_cache_miss() => {
                                            let creator = Creator::deleted(cached.base.creator);

                                            gdcf.cache().store_secondary(&creator.clone().into()).map_err(GdcfError::Cache)?;

                                            Ok(exchange::level_user(cached, creator))
                                        },

                                        Err(err) => Err(GdcfError::Cache(err)),
                                    }
                                }),
                        )
                    },

                    Err(err) => GdcfFuture::cache_error(err),
                }
            },

            GdcfFuture { cached, inner: Some(f) } => {
                let cached = match cached {
                    Some(cached) =>
                        match self.cache().lookup_creator(cached.inner().base.creator) {
                            Ok(creator) => Some(cached.map(|inner| exchange::level_user(inner, creator.extract()))),

                            Err(ref err) if err.is_cache_miss() => None, /* NOTE: here we cannot decide whether the creator isn't
                                                                           * cached, or */
                            // whether his GD account was deleted. We go with the conversative
                            // option and assume it wasn't cached.
                            Err(err) => return GdcfFuture::cache_error(err),
                        },

                    None => None,
                };

                if let Some(ref level) = cached {
                    info!("Cache entry is {}", level.inner());
                } else {
                    warn!("Cache entry for request missing");
                }

                GdcfFuture::new(
                    cached,
                    Some(f.and_then(move |level| {
                        let lookup = gdcf.cache().lookup_creator(level.base.creator);

                        match lookup {
                            Ok(creator) => Either::B(ok(exchange::level_user(level, creator.extract()))),

                            Err(ref err) if err.is_cache_miss() =>
                                Either::A(
                                    gdcf.levels::<u64, u64>(LevelsRequest::default().with_id(level.base.level_id))
                                        .and_then(move |_| {
                                            let lookup = gdcf.cache().lookup_creator(level.base.creator);

                                            match lookup {
                                                Ok(creator) => Ok(exchange::level_user(level, creator.extract())),

                                                Err(ref err) if err.is_cache_miss() => {
                                                    let creator = Creator::deleted(level.base.creator);

                                                    gdcf.cache().store_secondary(&creator.clone().into()).map_err(GdcfError::Cache)?;

                                                    Ok(exchange::level_user(level, creator))
                                                },

                                                Err(err) => Err(GdcfError::Cache(err)),
                                            }
                                        }),
                                ),

                            Err(error) => Either::B(err(GdcfError::Cache(error))),
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
    pub fn level<Song, User>(&self, request: LevelRequest) -> GdcfFuture<Level<Song, User>, A::Err, C::Err>
    where
        Self: ProcessRequest<A, C, LevelRequest, Level<Song, User>>,
        A: MakeRequest<LevelRequest>,
        C: CanCache<LevelRequest>,
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
    pub fn levels<Song, User>(&self, request: LevelsRequest) -> GdcfFuture<Vec<PartialLevel<Song, User>>, A::Err, C::Err>
    where
        Self: ProcessRequest<A, C, LevelsRequest, Vec<PartialLevel<Song, User>>>,
        A: MakeRequest<LevelsRequest>,
        C: CanCache<LevelsRequest>,
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
    pub fn user(&self, request: UserRequest) -> GdcfFuture<User, A::Err, C::Err>
    where
        A: MakeRequest<UserRequest>,
        C: CanCache<UserRequest>,
    {
        self.process_request(request)
    }
}

#[allow(missing_debug_implementations)]
pub struct GdcfFuture<T, A: ApiError, C: CacheError> {
    // invariant: at least one of the fields is not `None`
    pub cached: Option<CachedObject<T>>,
    pub inner: Option<Box<dyn Future<Item = T, Error = GdcfError<A, C>> + Send + 'static>>,
}

impl<T, A: ApiError, C: CacheError> GdcfFuture<T, A, C> {
    fn new<F>(cached: Option<CachedObject<T>>, f: Option<F>) -> Self
    where
        F: Future<Item = T, Error = GdcfError<A, C>> + Send + 'static,
    {
        GdcfFuture {
            cached,
            inner: match f {
                None => None,
                Some(f) => Some(Box::new(f)),
            },
        }
    }

    fn up_to_date(object: CachedObject<T>) -> Self {
        GdcfFuture {
            cached: Some(object),
            inner: None,
        }
    }

    fn outdated<F>(object: CachedObject<T>, f: F) -> Self
    where
        F: Future<Item = T, Error = GdcfError<A, C>> + Send + 'static,
    {
        GdcfFuture::new(Some(object), Some(f))
    }

    fn absent<F>(f: F) -> Self
    where
        F: Future<Item = T, Error = GdcfError<A, C>> + Send + 'static,
    {
        GdcfFuture::new(None, Some(f))
    }

    fn cache_error(error: C) -> Self
    where
        T: Send + 'static,
    {
        GdcfFuture::new(None, Some(err(GdcfError::Cache(error))))
    }

    fn error(error: GdcfError<A, C>) -> Self
    where
        T: Send + 'static,
    {
        GdcfFuture::new(None, Some(err(error)))
    }

    pub fn cached(&self) -> &Option<CachedObject<T>> {
        &self.cached
    }
}

impl<T, A: ApiError, C: CacheError> Future for GdcfFuture<T, A, C> {
    type Error = GdcfError<A, C>;
    type Item = T;

    fn poll(&mut self) -> Result<Async<T>, GdcfError<A, C>> {
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
    current_request: GdcfFuture<T, A::Err, C::Err>,
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
