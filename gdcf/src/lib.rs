#![feature(box_syntax)]
#![feature(attr_literals)]
#![feature(never_type)]

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
//! # The API Client
//!
//! The API Client used by GDCF must provide access to all data returning endpoints of the boomlings
//! API or an equivalent thereof. GDCF tries to abstract away as must as possible of RobTop's request
//! and response formatting, to allow the use of alternate Geometry Dash APIs, like GDJSAPI.
//! This also means GDCF does not include a default implementation of any client.
//!
//! # The data model
//!
//! GDCF models its data very closely to the way RobTop does. Due to the way RobTop's data is
//! organized, it is impossible to use frameworks like serde to process them. In GDCF you can either
//! choose to construct the objects yourself, or convert them to RobTop's indexed data format
//! and have GDCF construct them for you.
//!
//! # The Cache
//!
//! The only assumption GDCF makes about its cache is that if the `store_` methods return an `Ok(...)`
//! value, the data has been successfully cached. If your cache implementation chooses to not cache
//! a specific type of data, it must return an `Err(...)` value, otherwise GDCF cannot uphold its
//! integrity guarantees

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
use api::response::ProcessedResponse;
use cache::Cache;
use error::CacheError;
use ext::{ApiClientExt, CacheExt};
use futures::Future;
use futures::future::join_all;
use model::GDObject;
use std::sync::{Arc, Mutex, MutexGuard};

#[macro_use]
mod macros;
mod ext;

pub mod api;
pub mod cache;
pub mod model;
pub mod error;

pub trait Gdcf<A: ApiClient + 'static, C: Cache + 'static> {
    fn client(&self) -> MutexGuard<A>;
    fn cache(&self) -> MutexGuard<C>;

    fn refresh<R: Request + 'static>(&self, request: R);

    gdcf!(level, LevelRequest, lookup_level);
    gdcf!(levels, LevelsRequest, lookup_partial_levels);
}

pub struct CacheManager<A: ApiClient + 'static, C: Cache + 'static> {
    client: Arc<Mutex<A>>,
    cache: Arc<Mutex<C>>,
}

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

    fn store_result<F>(f: F, mutex: Arc<Mutex<C>>) -> impl Future<Item=(), Error=()>
        where
            F: Future<Item=ProcessedResponse, Error=()> + 'static
    {
        f.map(move |response| {
            lock!(mutex).store_all(response);
        })
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
        let request_string = format!("{}", request);
        let request_future = lock!(client_mutex).make(request);

        request_future.and_then(move |response| {
            let mut integrity_futures = Vec::new();

            for raw_object in &response {
                match Self::ensure_integrity(lock!(@cache_mutex), raw_object) {
                    Ok(Some(integrity_request)) => {
                        warn!("Integrity for result of {} is not given, making integrity request {}", request_string, integrity_request);

                        let future = Self::with_integrity(integrity_request, client_mutex.clone(), cache_mutex.clone());

                        integrity_futures.push(future);
                    }

                    Err(err) => {
                        return Err(error!("Error while constructing integrity request for {}: {:?}", request_string, err));
                    }

                    _ => ()
                }
            }

            if !integrity_futures.is_empty() {
                let integrity_future = join_all(integrity_futures)
                    .map(move |_| {
                        debug!("Successfully stored all data relevant for integrity!");
                        lock!(cache_mutex).store_all(response);
                    })
                    .map_err(move |_| error!("Failed to ensure integrity of {}'s result, not caching response!", request_string));

                lock!(client_mutex).spawn(integrity_future);
            } else {
                debug!("Result of {} does not compromise cache integrity, proceeding!", request_string);
                lock!(cache_mutex).store_all(response);
            }

            Ok(())
        })
    }
}

impl<A: ApiClient + 'static, C: Cache + 'static> Gdcf<A, C> for CacheManager<A, C> {
    fn client(&self) -> MutexGuard<A> {
        lock!(self.client)
    }

    fn cache(&self) -> MutexGuard<C> {
        lock!(self.cache)
    }

    fn refresh<R: Request + 'static>(&self, request: R) {
        info!("Cache entry for {} is either expired or non existant, refreshing!", request);

        let client = self.client();
        let future = Self::store_result(client.make(request), self.cache.clone());

        client.spawn(future);
    }
}

impl<A: ApiClient + 'static, C: Cache + 'static> Gdcf<A, C> for ConsistentCacheManager<A, C> {
    fn client(&self) -> MutexGuard<A> {
        lock!(self.client)
    }

    fn cache(&self) -> MutexGuard<C> {
        lock!(self.cache)
    }

    fn refresh<R: Request + 'static>(&self, request: R) {
        info!("Cache entry for {} is either expired or non existant, refreshing with integrity check!", request);

        let future = Self::with_integrity(request, self.client.clone(), self.cache.clone());

        self.client().spawn(future);
    }
}