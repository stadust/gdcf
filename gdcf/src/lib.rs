#![feature(box_syntax)]
#![feature(attr_literals)]
#![feature(never_type)]
#![feature(try_from)]

#![deny(
bare_trait_objects, missing_debug_implementations, unused_extern_crates, patterns_in_fns_without_body, stable_features, unknown_lints, unused_features, unused_imports, unused_parens
)]

//! The `gdcf` crate is the core of the Geometry Dash Caching Framework.
//! It provides all the core traits required to implement an API Client and
//! a cache which are used by implementations of the [`Gdcf`] trait.
//!
//! [`Gdcf`]: trait.Gdcf.html
//!
//! # Geometry Dash Caching Framework
//!
//! The idea behind the Geometry Dash Caching Framework is to provide fast and reliable
//! access to the resources provided by the Geometry Dash servers. It achieves this goal by caching all
//! responses from the servers and only returning those cached responses when a request is attempted,
//! while refreshing the cache asynchronously, in the background. This ensures instant access
//! to information such as level description that can be used easily even in environments where
//! the slow response times and unreliable availability of RobTop's server would be unacceptable otherwise
//!
//! It further has the ability to ensure the integrity of its cached data, which means it can
//! automatically generate more requests if it notices that, i.e., a level you just retrieved
//! doesn't have its newgrounds song cached.
//!
extern crate base64;
extern crate chrono;
extern crate futures;
#[macro_use]
extern crate gdcf_derive;
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

use api::client::ApiClient;
use api::request::{LevelRequest, LevelsRequest, Request};
use api::request::cacher::Cacher;
use cache::Cache;
use error::CacheError;
use ext::ApiClientExt;
use futures::Future;
use futures::future::join_all;
use model::GDObject;
use model::level::{Level, PartialLevel};
use std::sync::{Arc, Mutex, MutexGuard};

#[macro_use]
mod macros;

pub mod ext;
pub mod api;
pub mod cache;
pub mod model;
pub mod error;
pub mod convert;
mod gcdf2;
//mod selfless;

pub trait Gdcf<A: ApiClient + 'static, C: Cache + 'static> {
    fn cache(&self) -> MutexGuard<C>;

    fn refresh<R: Request + 'static>(&self, request: R);

    gdcf!(level, LevelRequest, lookup_level, Level);
    gdcf!(levels, LevelsRequest, lookup_partial_levels, Vec<PartialLevel>);
}

#[derive(Debug)]
pub struct CacheManager<A: ApiClient + 'static, C: Cache + 'static> {
    client: Arc<Mutex<A>>,
    cache: Arc<Mutex<C>>,
}

#[derive(Debug)]
pub struct ConsistentCacheManager<A: ApiClient + 'static, C: Cache + 'static> {
    client: Arc<Mutex<A>>,
    cache: Arc<Mutex<C>>,
}

impl<A: ApiClient + 'static, C: Cache + 'static> CacheManager<A, C> {
    pub fn new(client: A, cache: C) -> CacheManager<A, C> {
        info!("Created new CacheManager");

        CacheManager {
            client: Arc::new(Mutex::new(client)),
            cache: Arc::new(Mutex::new(cache)),
        }
    }
}

impl<A: ApiClient + 'static, C: Cache + 'static> ConsistentCacheManager<A, C> {
    pub fn new(client: A, cache: C) -> ConsistentCacheManager<A, C> {
        info!("Created new ConsistentCacheManager");

        ConsistentCacheManager {
            client: Arc::new(Mutex::new(client)),
            cache: Arc::new(Mutex::new(cache)),
        }
    }

    fn ensure_integrity(cache: &C, object: &GDObject) -> Result<Option<impl Request>, CacheError<C::Err>> {
        use api::request::level::SearchFilters;

        match *object {
            GDObject::Level(ref level) => {
                if let Some(song_id) = level.base.custom_song_id {
                    on_miss! {
                        cache.lookup_song(song_id) => {
                            Ok(Some(LevelsRequest::default()
                                .with_id(level.base.level_id)
                                .filter(SearchFilters::default()
                                    .custom_song(song_id))))
                        }
                    }
                } else {
                    Ok(None)
                }
            }
            _ => Ok(None)
        }
    }

    fn with_integrity<R>(request: R, client_mutex: Arc<Mutex<A>>, cache_mutex: Arc<Mutex<C>>) -> impl Future<Item=(), Error=()> + 'static
        where
            R: Request + 'static,
    {
        let request_future = lock!(client_mutex).make(&request);

        request_future.and_then(move |response| {
            let mut integrity_futures = Vec::new();

            for raw_object in &response {
                match Self::ensure_integrity(lock!(@cache_mutex), raw_object) {
                    Ok(Some(integrity_request)) => {
                        warn!("Integrity for result of {} is not given, making integrity request {}", request, integrity_request);

                        let future = Self::with_integrity(integrity_request, client_mutex.clone(), cache_mutex.clone());

                        integrity_futures.push(future);
                    }

                    Err(err) => {
                        return Err(error!("Error while constructing integrity request for {}: {:?}", request, err));
                    }

                    _ => ()
                }
            }

            if !integrity_futures.is_empty() {
                let req_str = request.to_string();
                let integrity_future = join_all(integrity_futures)
                    .map(move |_| {
                        debug!("Successfully stored all data relevant for integrity!");
                        R::ResponseCacher::store_response(lock!(!cache_mutex), &request, response)
                            .map_err(|err| error!("Failed to store response to request {}: {:?}", request, err));
                    })
                    .map_err(move |_| error!("Failed to ensure integrity of {}'s result, not caching response!", req_str));

                // TODO: this
                //lock!(client_mutex).spawn(integrity_future);
            } else {
                debug!("Result of {} does not compromise cache integrity, proceeding!", request);
                R::ResponseCacher::store_response(lock!(!cache_mutex), &request, response)
                    .map_err(|err| error!("Failed to store response to request {}: {:?}", request, err));
            }

            Ok(())
        })
    }
}

impl<A: ApiClient + 'static, C: Cache + 'static> Gdcf<A, C> for CacheManager<A, C> {
    fn cache(&self) -> MutexGuard<C> {
        lock!(self.cache)
    }

    fn refresh<R: Request + 'static>(&self, request: R) {
        info!("Cache entry for {} is either expired or non existant, refreshing!", request);

        let client = lock!(self.client);
        let cache = self.cache.clone();

        let future = client.make(&request)
            .map(move |response| {
                R::ResponseCacher::store_response(lock!(!cache), &request, response)
                    .map_err(|err| error!("Failed to store response to request {}: {:?}", request, err));
            });

        // TODO: this
        //client.spawn(future);
    }
}

impl<A: ApiClient + 'static, C: Cache + 'static> Gdcf<A, C> for ConsistentCacheManager<A, C> {
    fn cache(&self) -> MutexGuard<C> {
        lock!(self.cache)
    }

    fn refresh<R: Request + 'static>(&self, request: R) {
        info!("Cache entry for {} is either expired or non existant, refreshing with integrity check!", request);

        let future = Self::with_integrity(request, self.client.clone(), self.cache.clone());

        // TODO: this
        //lock!(self.client).spawn(future);
    }
}