#![feature(try_from)]
#![feature(box_syntax)]
#![feature(attr_literals)]
#![feature(never_type)]
#![feature(concat_idents)]

#[cfg(feature = "deser")]
#[macro_use]
extern crate serde_derive;
#[cfg(feature = "deser")]
extern crate serde;

#[macro_use]
extern crate lazy_static;
extern crate percent_encoding;
extern crate futures;
extern crate chrono;
#[macro_use]
extern crate log;

#[macro_use]
extern crate gdcf_derive;

use futures::Future;
use futures::future::join_all;

use cache::Cache;
use model::{FromRawObject, ObjectType, RawObject};

use api::client::ApiClient;
use api::GDError;
use api::request::{Request, MakeRequest, LevelsRequest, LevelRequest};

use ext::{ApiClientExt, CacheExt};

use std::sync::mpsc::{self, Sender, Receiver};
use std::sync::{Arc, Mutex, MutexGuard};

use std::thread;
use api::response::ProcessedResponse;

#[macro_use]
mod macros;
mod ext;

pub mod api;
pub mod cache;
pub mod model;

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

        let client = self.client();
        let client_mutex = self.client.clone();
        let cache_mutex = self.cache.clone();

        let request_string = format!("{}", request);

        let future = client.make(request)
            .and_then(move |response| {
                let client = &*client_mutex.lock().unwrap();
                let integrity_futures = {
                    let cache = &*cache_mutex.lock().unwrap();

                    let mut integrity_futures = Vec::new();

                    for raw_object in response.iter() {
                        match ensure_integrity(cache, raw_object) {
                            Ok(Some(integrity_request)) => {
                                warn!("Integrity for result of {} is not given, making integrity request {}", request_string, integrity_request);

                                integrity_futures.push(store_result(client.make(integrity_request), cache_mutex.clone()));
                            }

                            Err(err) => {
                                return Err(error!("Error while constructing integrity request for {}: {:?}", request_string, err));
                            }

                            _ => ()
                        }
                    }

                    integrity_futures
                };

                if !integrity_futures.is_empty() {
                    let integrity_future = join_all(integrity_futures)
                        .map(move |_| {
                            let mut cache = cache_mutex.lock().unwrap();

                            for raw_object in response.iter() {
                                cache.store_raw(raw_object);
                            }
                        })
                        .map_err(move |_| error!("Failed to ensure integrity of {}'s result, not caching response!", request_string));

                    client.spawn(integrity_future);
                } else {
                    debug!("Result of {} does not compromise cache integrity, proceeding!", request_string);

                    for raw_object in response.iter() {
                        cache_mutex.lock().unwrap().store_raw(raw_object);
                    }
                }

                Ok(())
            });

        client.spawn(future);
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

fn ensure_integrity<C: Cache>(cache: &C, raw: &RawObject) -> Result<Option<impl MakeRequest>, GDError> {
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