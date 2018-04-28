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
//! ## Requests
//!
//! GDCF provides structs modelling all requests that can be made to the boomlings API in the
//! [`request`] module. For further information on how to implement endpoints of custom
//! APIs and use them with GDCF, see the documentation of that module
//!
//! [`request`]: api/request/index.html
//!
//! ## The data model
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
use api::request::{LevelRequest, LevelsRequest, MakeRequest, Request};
use api::response::ProcessedResponse;
use cache::Cache;
use error::GdcfError;
use ext::{ApiClientExt, CacheExt};
use futures::Future;
use futures::future::join_all;
use model::{FromRawObject, ObjectType, RawObject};
use std::error::Error;
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

    fn refresh<R: MakeRequest + 'static>(&self, request: R);

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
}

impl<A: ApiClient + 'static, C: Cache + 'static> ConsistentCacheManager<A, C> {
    pub fn new(client: A, cache: C) -> ConsistentCacheManager<A, C> {
        info!("Created new ConsistentCacheManager");

        ConsistentCacheManager {
            client: Arc::new(Mutex::new(client)),
            cache: Arc::new(Mutex::new(cache)),
        }
    }

    fn ensure_integrity(cache: &C, raw: &RawObject) -> Result<Option<impl MakeRequest>, GdcfError<A::Err, C::Err>> {
        use api::request::level::SearchFilters;

        match raw.object_type {
            ObjectType::Level => {
                let song_id: u64 = raw.get(35)?;

                if song_id != 0 {
                    if cache.lookup_song(song_id).is_none() {
                        Ok(Some(LevelsRequest::default()
                            .search(raw.get(1)?)
                            .filter(SearchFilters::default()
                                .custom_song(song_id))))
                    } else {
                        Ok(None)
                    }
                } else {
                    Ok(None)
                }
            }
            _ => Ok(None)
        }
    }
}

impl<A: ApiClient + 'static, C: Cache + 'static> Gdcf<A, C> for CacheManager<A, C> {
    fn client(&self) -> MutexGuard<A> {
        self.client.lock().unwrap()
    }

    fn cache(&self) -> MutexGuard<C> {
        self.cache.lock().unwrap()
    }

    fn refresh<R: MakeRequest + 'static>(&self, request: R) {
        info!("Cache entry for {} is either expired or non existant, refreshing!", request);

        let client = self.client();
        let future = store_result(client.make(request), self.cache.clone());

        client.spawn(future);
    }
}

impl<A: ApiClient + 'static, C: Cache + 'static> Gdcf<A, C> for ConsistentCacheManager<A, C> {
    fn client(&self) -> MutexGuard<A> {
        self.client.lock().unwrap()
    }

    fn cache(&self) -> MutexGuard<C> {
        self.cache.lock().unwrap()
    }

    fn refresh<R: MakeRequest + 'static>(&self, request: R) {
        info!("Cache entry for {} is either expired or non existant, refreshing with integrity check!", request);

        let future = with_integrity(request, self.client.clone(), self.cache.clone());

        self.client().spawn(future);
    }
}

fn store_result<F: Future<Item=ProcessedResponse, Error=()> + 'static, C: Cache>(f: F, mutex: Arc<Mutex<C>>) -> impl Future<Item=(), Error=()> {
    f.map(move |response| {
        let mut cache = mutex.lock().unwrap();

        for raw_object in response.iter() {
            cache.store_raw(raw_object);
        }
    })
}

fn with_integrity<A, C, R>(request: R, client_mutex: Arc<Mutex<A>>, cache_mutex: Arc<Mutex<C>>) -> impl Future<Item=(), Error=()> + 'static
    where
        R: MakeRequest + 'static,
        A: ApiClient + 'static,
        C: Cache + 'static
{

    let request_string = format!("{}", request);
    let request_future = lock!(client_mutex).make(request);

    request_future.and_then(move |response| {
        let mut integrity_futures = Vec::new();

        for raw_object in response.iter() {
            match ConsistentCacheManager::<A, C>::ensure_integrity(lock!(@cache_mutex), raw_object) {
                Ok(Some(integrity_request)) => {
                    warn!("Integrity for result of {} is not given, making integrity request {}", request_string, integrity_request);

                    let future = with_integrity(integrity_request, client_mutex.clone(), cache_mutex.clone());

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
                    lock!(cache_mutex).store_all(response.iter());
                })
                .map_err(move |_| error!("Failed to ensure integrity of {}'s result, not caching response!", request_string));

            lock!(client_mutex).spawn(integrity_future);
        } else {
            debug!("Result of {} does not compromise cache integrity, proceeding!", request_string);
            lock!(cache_mutex).store_all(response.iter());
        }

        Ok(())
    })
}