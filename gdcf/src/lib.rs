#![feature(try_from)]
#![feature(box_syntax)]
#![feature(attr_literals)]
#![feature(never_type)]
#![feature(concat_idents)]

#[cfg(ser)]
#[macro_use]
extern crate serde_derive;
#[cfg(ser)]
extern crate serde;

#[macro_use]
extern crate lazy_static;
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

use std::sync::mpsc::{self, Sender, Receiver};
use std::sync::{Arc, Mutex, MutexGuard};

use std::thread;
use api::response::ProcessedResponse;

#[macro_use]
mod macros;
pub mod api;
pub mod cache;
pub mod model;

pub struct Gdcf<A: 'static, C: 'static>
    where
        A: ApiClient,
        C: Cache + Send,
{
    cache: Arc<Mutex<C>>,
    client: Arc<Mutex<A>>,
    sender: Sender<RawObject>,
}

impl<A: 'static, C: 'static> Gdcf<A, C>
    where
        A: ApiClient,
        C: Cache + Send,
{
    pub fn new(cache: C, client: A) -> Gdcf<A, C> {
        debug!("Created new GDCF");

        let (tx, rx): (Sender<RawObject>, Receiver<RawObject>) = mpsc::channel();
        let cache_mutex = Arc::new(Mutex::new(cache));
        let client_mutex = Arc::new(Mutex::new(client));

        let handle = {
            let mutex = Arc::clone(&cache_mutex);

            thread::spawn(move || {
                info!("Started background cache manager thread");

                for raw_obj in rx {
                    let mut cache = mutex.lock().unwrap();

                    debug!("Received a {:?}, attempting to cache", raw_obj.object_type);

                    let err = match raw_obj.object_type {
                        ObjectType::Level => store!(cache, store_level, raw_obj),
                        ObjectType::PartialLevel => store!(cache, store_partial_level, raw_obj),
                        ObjectType::NewgroundsSong => store!(cache, store_song, raw_obj)
                    };

                    if let Err(err) = err {
                        error!(
                            "Unexpected error while constructing object {:?}: {:?}",
                            raw_obj.object_type, err
                        )
                    }
                }
            })
        };

        Gdcf {
            cache: cache_mutex,
            client: client_mutex,
            sender: tx,
        }
    }

    pub fn cache(&self) -> MutexGuard<C> {
        self.cache.lock().unwrap()
    }

    pub fn client(&self) -> MutexGuard<A> { self.client.lock().unwrap() }

    gdcf!(level, LevelRequest, lookup_level);
    gdcf!(levels, LevelsRequest, lookup_partial_levels);

    #[cfg(not(feature = "ensure_cache_integrity"))]
    fn refresh<R: MakeRequest + 'static>(&self, req: R) {
        let future = req.make(&*self.client());

        self.client().spawn(store(self.sender.clone(), future));
    }


    #[cfg(feature = "ensure_cache_integrity")]
    fn refresh<R: MakeRequest + 'static>(&self, req: R) {
        info!("Cache entry for {} is either expired or non existant, refreshing!", req);

        let cache = Arc::clone(&self.cache);
        let client = Arc::clone(&self.client);
        let sender = self.sender.clone();

        let future = req.make(&*self.client())
            .and_then(move |resp| {
                let cache = &*cache.lock().unwrap();
                let client = &*client.lock().unwrap();

                let integrity_futures = match resp {
                    ProcessedResponse::One(ref obj) => {
                        if let Some(ireq) = ensure_integrity(cache, obj)? {
                            warn!("Integrity for result of {} is not given, making integrity request {}", req, ireq);

                            vec![store(sender.clone(), ireq.make(client))]
                        } else {
                            Vec::new()
                        }
                    },
                    ProcessedResponse::Many(ref objs) => {
                        let mut futures = Vec::new();

                        for obj in objs {
                            if let Some(ireq) = ensure_integrity(cache, obj)? {
                                warn!("Integrity for result of {} is not given, making integrity request {}", req, ireq);

                                futures.push(store(sender.clone(), ireq.make(client)))
                            }
                        }

                        futures
                    }
                };

                if !integrity_futures.is_empty() {
                    let future = join_all(integrity_futures)
                        .map(move |_| send(&sender, resp));

                    client.spawn(future);
                } else {
                    debug!("Result of {} does not compromise cache integrity, proceeding!", req);
                    send(&sender, resp);
                }

                Ok(())
            })
            .map_err(|e| error!("Unexpected error while retrieving integrity data for cache: {:?}", e));

        self.client().spawn(future);
    }
}

fn send(sender: &Sender<RawObject>, resp: ProcessedResponse) {
    match resp {
        ProcessedResponse::One(obj) => sender.send(obj).unwrap(), // TODO: error
        ProcessedResponse::Many(objs) => {
            for obj in objs {
                sender.send(obj).unwrap() // TODO: error
            }
        }
    }
}

fn store<F>(sender: Sender<RawObject>, f: F) -> impl Future<Item=(), Error=()> + 'static
    where
        F: Future<Item=ProcessedResponse, Error=GDError> + 'static
{
    f.map(move |resp| send(&sender.clone(), resp))
        .map_err(|e| error!("Unexpected error while retrieving data for cache: {:?}", e))
}


#[cfg(feature = "ensure_cache_integrity")]
fn ensure_integrity<C: Cache>(cache: &C, raw: &RawObject) -> Result<Option<impl MakeRequest>, GDError> {
    use api::request::level::SearchFilters;

    match raw.object_type {
        ObjectType::Level => {
            let song_id: u64 = raw.get(35)?;

            if song_id != 0 {
                let existing = cache.lookup_song(song_id);

                if existing.is_none() {
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
        },
        _ => Ok(None)
    }
}