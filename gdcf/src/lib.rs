#![feature(try_from)]
#![feature(box_syntax)]
#![feature(attr_literals)]
#![feature(never_type)]
#![feature(concat_idents)]

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
use api::GDError;
use api::request::{LevelRequest, LevelsRequest, MakeRequest, Request};
use api::response::ProcessedResponse;
use cache::Cache;
use ext::{ApiClientExt, CacheExt};
use futures::Future;
use futures::future::join_all;
use model::{FromRawObject, ObjectType, RawObject};
use std::sync::{Arc, Mutex, MutexGuard};

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

        self.client().spawn(with_integrity(request, self.client.clone(), self.cache.clone()));
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
    let request_future = client_mutex.lock().unwrap().make(request);

    request_future.and_then(move |response| {
        let mut integrity_futures = Vec::new();

        for raw_object in response.iter() {
            match ensure_integrity(lock!(@cache_mutex), raw_object) {
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