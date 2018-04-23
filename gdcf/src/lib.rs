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

use cache::Cache;
use model::{FromRawObject, ObjectType, RawObject};

use api::client::ApiClient;
use api::GDError;
use api::request::{Request, MakeRequest, LevelsRequest, LevelRequest};

use std::sync::mpsc::{self, Sender, Receiver};
use std::sync::{Arc, Mutex, MutexGuard};

use std::thread;

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

    retrieve_one!(level, LevelRequest, lookup_level);
    retrieve_many!(levels, LevelsRequest, lookup_partial_levels);
}

fn store_one<F>(sender: Sender<RawObject>, future: F) -> impl Future<Item=(), Error=()> + 'static
    where
        F: Future<Item=RawObject, Error=GDError> + 'static,
{
    future
        .map(move |obj| sender.send(obj).unwrap())
        .map_err(|e| error!("Unexpected error while retrieving data for cache: {:?}", e))
}

fn store_many<F>(sender: Sender<RawObject>, future: F) -> impl Future<Item=(), Error=()> + 'static
    where
        F: Future<Item=Vec<RawObject>, Error=GDError> + 'static,
{
    future
        .map(move |objs| objs.into_iter().for_each(|obj| sender.send(obj).unwrap()))
        .map_err(|e| error!("Unexpected error while retrieving data for cache: {:?}", e))
}

#[cfg(feature = "ensure_cache_integrity")]
fn level_integrity<C: Cache>(cache: &C, raw_level: &RawObject) -> Result<Option<LevelsRequest>, GDError> {
    use api::request::level::SearchFilters;

    let song_id: u64 = raw_level.get(35)?;

    if song_id != 0 {
        let existing = cache.lookup_song(song_id);

        if existing.is_none() {
            Ok(Some(LevelsRequest::default()
                .search(raw_level.get(1)?)
                .filter(SearchFilters::default()
                    .custom_song(song_id))))
        } else {
            Ok(None)
        }
    } else {
        Ok(None)
    }
}