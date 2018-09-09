#![feature(box_syntax)]
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
//! a cache which are used by implementations of the [`Gdcf`] trait.
//!
//! [`Gdcf`]: trait.Gdcf.html
//!
//! # Geometry Dash Caching Framework
//!
//! The idea behind the Geometry Dash Caching Framework is to provide fast and
//! reliable access to the resources provided by the Geometry Dash servers. It
//! achieves this goal by caching all responses from the servers and only
//! returning those cached responses when a
//! request is attempted, while refreshing the cache asynchronously, in the
//! background. This ensures instant access to information such as level
//! description that can be used easily
//! even in environments where the slow response times and unreliable
//! availability of RobTop's server would be
//! unacceptable otherwise
//!
//! It further ensures the integrity of its cached data, which means it
//! automatically generates more requests if it notices that, i.e., a level you
//! just retrieved doesn't have its newgrounds song
//! cached.
//!
extern crate base64;
extern crate chrono;
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

use api::{
    request::{level::SearchFilters, LevelRequest, LevelsRequest, PaginatableRequest, Request},
    ApiClient,
};
use cache::Cache;
use error::{ApiError, CacheError, GdcfError};
use futures::{
    future::{result, Either, FutureResult},
    task, Async, Future, Stream,
};
use model::{GDObject, Level, NewgroundsSong, PartialLevel};
use std::{
    error::Error,
    mem,
    sync::{Arc, Mutex, MutexGuard},
};

#[macro_use]
mod macros;

pub mod api;
mod build;
pub mod cache;
pub mod convert;
pub mod error;
pub mod model;

// TODO: for levels, get their creator via the getGJProfile endpoint, then we can give PartialLevel
// a User

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

#[derive(Debug)]
pub struct Gdcf<A: ApiClient + 'static, C: Cache + 'static> {
    client: Arc<Mutex<A>>,
    cache: Arc<Mutex<C>>,
}

impl<A: ApiClient + 'static, C: Cache + 'static> Clone for Gdcf<A, C> {
    fn clone(&self) -> Self {
        Gdcf {
            client: self.client.clone(),
            cache: self.cache.clone(),
        }
    }
}

impl<A: ApiClient, C: Cache> ProcessRequest<A, C, LevelRequest, Level<u64>> for Gdcf<A, C> {
    fn process_request(&self, request: LevelRequest) -> GdcfFuture<Level<u64>, A::Err, C::Err> {
        info!("Processing request {} with 'u64' as Song type", request);

        gdcf! {
            self, request, lookup_level, || {
                let cache = self.cache.clone();

                self.client().level(request)
                    .map_err(GdcfError::Api)
                    .and_then(collect_one!(cache, Level))
            }
        }
    }
}

impl<A: ApiClient, C: Cache> ProcessRequest<A, C, LevelRequest, Level<NewgroundsSong>> for Gdcf<A, C> {
    fn process_request(&self, request: LevelRequest) -> GdcfFuture<Level<NewgroundsSong>, A::Err, C::Err> {
        info!("Processing request {} with 'NewgroundsSong' as Song type", request);

        let raw: GdcfFuture<Level<u64>, _, _> = self.process_request(request);
        let cache = self.cache.clone();

        // TODO: reintroduce debugging statements

        match raw {
            GdcfFuture {
                cached: Some(cached),
                inner: None,
            } =>
                match cached.base.custom_song {
                    None => GdcfFuture::up_to_date(build::build_level(cached, None)),
                    Some(custom_song_id) => {
                        // We cannot do the lookup in the match because then the cache would be locked for the entire match
                        // block which would deadlock because of the `process_request` call in it.
                        let lookup = self.cache().lookup_song(custom_song_id);

                        match lookup {
                            Ok(song) => GdcfFuture::up_to_date(build::build_level(cached, Some(song.extract()))),

                            Err(CacheError::CacheMiss) => {
                                warn!("The level requested was cached, but not its song, performing a request to retrieve it!");

                                GdcfFuture::absent(
                                    self.process_request(
                                        LevelsRequest::default()
                                            .with_id(cached.base.level_id)
                                            .filter(SearchFilters::default().custom_song(custom_song_id)),
                                    ).and_then(move |_: Vec<PartialLevel<u64>>| {
                                        let cache = cache.lock().unwrap();

                                        Ok(build::build_level(cached, Some(cache.lookup_song(custom_song_id)?.extract())))
                                    }),
                                )
                            },

                            Err(err) => GdcfFuture::error(err),
                        }
                    },
                },

            GdcfFuture { cached, inner: Some(f) } => {
                let cached = match cached {
                    Some(cached) =>
                        match cached.base.custom_song {
                            None => Some(build::build_level(cached, None)),
                            Some(custom_song_id) =>
                                match self.cache().lookup_song(custom_song_id) {
                                    Ok(song) => Some(build::build_level(cached, Some(song.extract()))),

                                    Err(CacheError::CacheMiss) => None,

                                    Err(err) => return GdcfFuture::error(err),
                                },
                        },
                    None => None,
                };

                let gdcf = self.clone();

                GdcfFuture::new(
                    cached,
                    Some(f.and_then(move |level| {
                        if let Some(song_id) = level.base.custom_song {
                            // We cannot do the lookup in the match because then the cache would be locked for the entire match
                            // block which would deadlock because of the `process_request` call in it.
                            let lookup = gdcf.cache().lookup_song(song_id);

                            match lookup {
                                Err(CacheError::CacheMiss) => {
                                    warn!(
                                        "The level the song for the requested level was not cached, performing a request to retrieve it!"
                                    );

                                    Either::A(
                                        gdcf.process_request(
                                            LevelsRequest::default()
                                                .with_id(level.base.level_id)
                                                .filter(SearchFilters::default().custom_song(song_id)),
                                        ).and_then(move |_: Vec<PartialLevel<u64>>| {
                                            let cache = cache.lock().unwrap();

                                            Ok(build::build_level(level, Some(cache.lookup_song(song_id)?.extract())))
                                        }),
                                    )
                                },

                                Err(err) => Either::B(result(Err(GdcfError::Cache(err)))),

                                Ok(song) => Either::B(result(Ok(build::build_level(level, Some(song.extract()))))),
                            }
                        } else {
                            Either::B(result(Ok(build::build_level(level, None))))
                        }
                    })),
                )
            },

            _ => unreachable!(),
        }
    }
}

impl<A: ApiClient, C: Cache> ProcessRequest<A, C, LevelsRequest, Vec<PartialLevel<u64>>> for Gdcf<A, C> {
    fn process_request(&self, request: LevelsRequest) -> GdcfFuture<Vec<PartialLevel<u64>>, A::Err, C::Err> {
        info!("Processing request {} with 'u64' as Song type", request);

        gdcf! {
            self, request, lookup_partial_levels, || {
                let cache = self.cache.clone();

                self.client().levels(request.clone())
                    .map_err(GdcfError::Api)
                    .and_then(collect_many!(request, cache, store_partial_levels, PartialLevel))
            }
        }
    }
}

impl<A: ApiClient, C: Cache> ProcessRequest<A, C, LevelsRequest, Vec<PartialLevel<NewgroundsSong>>> for Gdcf<A, C> {
    fn process_request(&self, request: LevelsRequest) -> GdcfFuture<Vec<PartialLevel<NewgroundsSong>>, A::Err, C::Err> {
        info!("Processing request {} with 'NewgroundsSong' as Song type", request);

        let GdcfFuture { cached, inner } = self.process_request(request);
        let cache = self.cache.clone();

        let processor = move |levels: Vec<PartialLevel<u64>>| {
            let cache = cache.lock().unwrap();
            let mut vec = Vec::new();

            for partial_level in levels {
                let built = match partial_level.custom_song {
                    Some(custom_song_id) =>
                        match cache.lookup_song(custom_song_id) {
                            Ok(song) => build::build_partial_level(partial_level, Some(song.extract())),

                            Err(CacheError::CacheMiss) => unreachable!(),

                            Err(err) => return Err(err.into()),
                        },

                    None => build::build_partial_level(partial_level, None),
                };

                vec.push(built);
            }

            Ok(vec)
        };

        let cached = match cached {
            Some(cached) =>
                match processor(cached) {
                    Ok(cached) => Some(cached),
                    Err(err) => return GdcfFuture::error(err),
                },
            None => None,
        };

        GdcfFuture::new(cached, inner.map(|fut| fut.and_then(processor)))
    }
}

// TODO: figure out a better way for the building stuff

impl<A: ApiClient + 'static, C: Cache + 'static> Gdcf<A, C> {
    pub fn new(client: A, cache: C) -> Gdcf<A, C> {
        Gdcf {
            client: Arc::new(Mutex::new(client)),
            cache: Arc::new(Mutex::new(cache)),
        }
    }

    pub fn cache(&self) -> MutexGuard<C> {
        self.cache.lock().unwrap()
    }

    pub fn client(&self) -> MutexGuard<A> {
        self.client.lock().unwrap()
    }

    pub fn level<Song>(&self, request: LevelRequest) -> impl Future<Item = Level<Song>, Error = GdcfError<A::Err, C::Err>>
    where
        Self: ProcessRequest<A, C, LevelRequest, Level<Song>>,
        Song: PartialEq,
    {
        self.process_request(request)
    }

    pub fn levels<Song>(&self, request: LevelsRequest) -> impl Future<Item = Vec<PartialLevel<Song>>, Error = GdcfError<A::Err, C::Err>>
    where
        Self: ProcessRequest<A, C, LevelsRequest, Vec<PartialLevel<Song>>>,
        Song: PartialEq,
    {
        self.process_request(request)
    }

    pub fn paginate_levels<Song>(
        &self, request: LevelsRequest,
    ) -> impl Stream<Item = Vec<PartialLevel<Song>>, Error = GdcfError<A::Err, C::Err>>
    where
        Self: ProcessRequest<A, C, LevelsRequest, Vec<PartialLevel<Song>>>,
        Song: PartialEq,
    {
        self.paginate(request)
    }
}

#[allow(missing_debug_implementations)]
pub struct GdcfFuture<T, AE: Error + Send + 'static, CE: Error + Send + 'static> {
    // invariant: at least one of the fields is not `None`
    cached: Option<T>,
    inner: Option<Box<dyn Future<Item = T, Error = GdcfError<AE, CE>> + Send + 'static>>,
}

impl<T, CE: Error + Send + 'static, AE: Error + Send + 'static> GdcfFuture<T, AE, CE> {
    fn new<F>(cached: Option<T>, f: Option<F>) -> GdcfFuture<T, AE, CE>
    where
        F: Future<Item = T, Error = GdcfError<AE, CE>> + Send + 'static,
    {
        GdcfFuture {
            cached,
            inner: match f {
                None => None,
                Some(f) => Some(Box::new(f)),
            },
        }
    }

    fn up_to_date(object: T) -> GdcfFuture<T, AE, CE> {
        GdcfFuture {
            cached: Some(object),
            inner: None,
        }
    }

    fn outdated<F>(object: T, f: F) -> GdcfFuture<T, AE, CE>
    where
        F: Future<Item = T, Error = GdcfError<AE, CE>> + Send + 'static,
    {
        GdcfFuture::new(Some(object), Some(f))
    }

    fn absent<F>(f: F) -> GdcfFuture<T, AE, CE>
    where
        F: Future<Item = T, Error = GdcfError<AE, CE>> + Send + 'static,
    {
        GdcfFuture::new(None, Some(f))
    }

    fn error<E: Into<GdcfError<AE, CE>>>(error: E) -> Self
    where
        T: Send + 'static,
    {
        GdcfFuture::new(None, Some(result(Err(error.into()))))
    }

    pub fn cached(&self) -> &Option<T> {
        &self.cached
    }

    pub fn take(&mut self) -> Option<T> {
        self.cached.take()
    }

    pub fn has_inner(&self) -> bool {
        self.inner.is_some()
    }

    pub fn into_inner(self) -> Box<dyn Future<Item = T, Error = GdcfError<AE, CE>> + Send + 'static> {
        self.inner.unwrap()
    }
}

impl<T, AE: Error + Send + 'static, CE: Error + Send + 'static> Future for GdcfFuture<T, AE, CE> {
    type Error = GdcfError<AE, CE>;
    type Item = T;

    fn poll(&mut self) -> Result<Async<T>, GdcfError<AE, CE>> {
        match self.inner {
            Some(ref mut fut) => fut.poll(),
            None => Ok(Async::Ready(self.take().unwrap())),
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
