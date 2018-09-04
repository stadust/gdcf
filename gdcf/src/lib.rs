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
    request::{level::SearchFilters, LevelRequest, LevelsRequest, StreamableRequest},
    ApiClient,
};
use cache::Cache;
use error::{ApiError, CacheError, GdcfError};
use futures::{
    future::{result, Either},
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

// TODO: find better names for the methods

// TODO: figure out a better way for the building stuff

impl<A: ApiClient + 'static, C: Cache + 'static> Gdcf<A, C> {
    gdcf_one!(level, lookup_level, LevelRequest, Level<u64>, Level);

    gdcf_many!(
        levels,
        lookup_partial_levels,
        store_partial_levels,
        LevelsRequest,
        PartialLevel<u64>,
        PartialLevel
    );

    stream!(levels, levelss, LevelsRequest, Vec<PartialLevel<u64>>);

    stream!(levels2, levels2s, LevelsRequest, Vec<PartialLevel<NewgroundsSong>>);

    pub fn new(client: A, cache: C) -> Gdcf<A, C> {
        Gdcf {
            client: Arc::new(Mutex::new(client)),
            cache: Arc::new(Mutex::new(cache)),
        }
    }

    pub fn level2(
        &self, request: LevelRequest,
    ) -> impl Future<Item = Level<NewgroundsSong>, Error = GdcfError<A::Err, C::Err>> + Send + 'static {
        let future = self.level(request);
        let gdcf = self.clone();
        let cache = self.cache.clone();

        future
            .and_then(move |level| {
                if let Some(song_id) = level.base.custom_song {
                    match gdcf.cache().lookup_song(song_id) {
                        Err(CacheError::CacheMiss) => {
                            warn!("Integrity request required to gather newgrounds song with ID {}", song_id);

                            Either::A(
                                gdcf.levels(
                                    LevelsRequest::default()
                                        .with_id(level.base.level_id)
                                        .filter(SearchFilters::default().custom_song(song_id)),
                                ).map(move |_| level),
                            )
                        },

                        Err(err) => Either::B(result(Err(GdcfError::Cache(err)))),

                        Ok(_) => Either::B(result(Ok(level))),
                    }
                } else {
                    Either::B(result(Ok(level)))
                }
            }).and_then(move |level| {
                let cache = cache.lock().unwrap();
                build::build_level(level, &*cache).map_err(GdcfError::Cache)
            })
    }

    pub fn levels2(
        &self, request: LevelsRequest,
    ) -> impl Future<Item = Vec<PartialLevel<NewgroundsSong>>, Error = GdcfError<A::Err, C::Err>> + Send + 'static {
        let future = self.levels(request);
        let cache = self.cache.clone();

        future.and_then(move |levels| {
            let mut built_levels = Vec::new();
            let cache = cache.lock().unwrap();

            for level in levels {
                built_levels.push(build::build_partial_level(level, &*cache)?)
            }

            Ok(built_levels)
        })
    }

    pub fn cache(&self) -> MutexGuard<C> {
        self.cache.lock().unwrap()
    }

    pub fn client(&self) -> MutexGuard<A> {
        self.client.lock().unwrap()
    }
}

#[allow(missing_debug_implementations)]
pub struct GdcfFuture<T, AE: Error + Send + 'static, CE: Error + Send + 'static> {
    // invariant: at least one of the fields is not `None`
    cached: Option<T>,
    refresher: Option<Box<dyn Future<Item = T, Error = GdcfError<AE, CE>> + Send + 'static>>,
}

impl<T, CE: Error + Send + 'static, AE: Error + Send + 'static> GdcfFuture<T, AE, CE> {
    fn up_to_date(object: T) -> GdcfFuture<T, AE, CE> {
        GdcfFuture {
            cached: Some(object),
            refresher: None,
        }
    }

    fn outdated<F>(object: T, f: F) -> GdcfFuture<T, AE, CE>
    where
        F: Future<Item = T, Error = GdcfError<AE, CE>> + Send + 'static,
    {
        GdcfFuture {
            cached: Some(object),
            refresher: Some(Box::new(f)),
        }
    }

    fn absent<F>(f: F) -> GdcfFuture<T, AE, CE>
    where
        F: Future<Item = T, Error = GdcfError<AE, CE>> + Send + 'static,
    {
        GdcfFuture {
            cached: None,
            refresher: Some(Box::new(f)),
        }
    }

    pub fn cached(&self) -> &Option<T> {
        &self.cached
    }

    pub fn take(&mut self) -> Option<T> {
        self.cached.take()
    }
}

impl<T, AE: Error + Send + 'static, CE: Error + Send + 'static> Future for GdcfFuture<T, AE, CE> {
    type Error = GdcfError<AE, CE>;
    type Item = T;

    fn poll(&mut self) -> Result<Async<T>, GdcfError<AE, CE>> {
        match self.refresher {
            Some(ref mut fut) => fut.poll(),
            None => Ok(Async::Ready(self.take().unwrap())),
        }
    }
}

// FIXME: This struct is just a huge WTF
#[allow(missing_debug_implementations)]
pub struct GdcfStream<S, T, AE, CE, F, Fut>
where
    S: StreamableRequest,
    AE: Error + Send + 'static,
    CE: Error + Send + 'static,
    Fut: Future<Item = T, Error = GdcfError<AE, CE>> + Send + 'static,
    F: Fn(S) -> Fut + Send,
{
    request: S,
    current: Fut,
    request_maker: F,
}

impl<S, T, AE, CE, F, Fut> GdcfStream<S, T, AE, CE, F, Fut>
where
    S: StreamableRequest,
    AE: Error + Send + 'static,
    CE: Error + Send + 'static,
    Fut: Future<Item = T, Error = GdcfError<AE, CE>> + Send + 'static,
    F: Fn(S) -> Fut + Send,
{
    pub fn new(request: S, func: F) -> GdcfStream<S, T, AE, CE, F, Fut> {
        let next = request.next();
        let current = func(request);

        GdcfStream {
            request: next,
            current,
            request_maker: func,
        }
    }
}

impl<S, T, AE, CE, F, Fut> Stream for GdcfStream<S, T, AE, CE, F, Fut>
where
    S: StreamableRequest,
    AE: Error + Send + 'static,
    CE: Error + Send + 'static,
    Fut: Future<Item = T, Error = GdcfError<AE, CE>> + Send + 'static,
    F: Fn(S) -> Fut + Send,
{
    type Error = GdcfError<AE, CE>;
    type Item = T;

    fn poll(&mut self) -> Result<Async<Option<T>>, GdcfError<AE, CE>> {
        match self.current.poll() {
            Ok(Async::NotReady) => Ok(Async::NotReady),

            Ok(Async::Ready(result)) => {
                task::current().notify();

                let next = self.request.next();
                let cur = mem::replace(&mut self.request, next);

                self.current = (self.request_maker)(cur);

                Ok(Async::Ready(Some(result)))
            },

            Err(GdcfError::NoContent) | Err(GdcfError::Api(ApiError::NoData)) => {
                info!("Stream over request {} terminating due to exhaustion!", self.request);

                Ok(Async::Ready(None))
            },

            Err(err) => Err(err),
        }
    }
}
