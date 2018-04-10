#![feature(try_from)]
#![feature(box_syntax)]
#![feature(attr_literals)]

#[macro_use]
extern crate serde_derive;
extern crate futures;
extern crate tokio_core;
extern crate serde;
#[macro_use]
extern crate gdcf_derive;

use futures::Future;

use cache::Cache;
use model::Level;
use model::GDObject;

use api::client::GDClient;
use api::request::level::LevelRequest;
use api::GDError;

use std::sync::mpsc;
use std::sync::mpsc::Sender;
use std::thread;
use std::sync::Mutex;
use std::sync::Arc;
use std::sync::MutexGuard;

pub mod cache;
pub mod model;
pub mod api;

pub struct Gdcf<A, C: 'static>
    where
        A: GDClient,
        C: Cache + Send
{
    cache: Arc<Mutex<C>>,
    client: A,
    sender: Sender<GDObject>,
}

impl<A, C: 'static> Gdcf<A, C>
    where
        A: GDClient,
        C: Cache + Send
{
    pub fn new(cache: C, client: A) -> Gdcf<A, C> {
        let (tx, rx) = mpsc::channel();
        let mutex = Arc::new(Mutex::new(cache));

        let handle = {
            let mutex = Arc::clone(&mutex);

            thread::spawn(move || {
                for gdobj in rx {
                    let mut cache = mutex.lock().unwrap();

                    match gdobj {
                        GDObject::Level(level) => cache.store_level(level)
                    }
                }
            })
        };

        Gdcf {
            cache: mutex,
            client,
            sender: tx,
        }
    }

    pub fn cache(&self) -> MutexGuard<C> {
        self.cache.lock().unwrap()
    }

    pub fn level(&self, lid: u64) -> Option<Level>
    {
        let (cached, expired) = {
            let cache = self.cache();
            let cached = cache.lookup_level(lid);
            let expired = cached.as_ref()
                .map_or(true, |co| cache.is_expired(co));

            (cached, expired)
        };

        if expired {
            self.refresh_one(self.client.level(LevelRequest::new(lid)));
        }

        cached.map(|co| co.extract())
    }

    fn refresh_one<F>(&self, future: F)
        where
            F: Future<Item=GDObject, Error=GDError> + 'static
    {
        let sender = self.sender.clone();
        let future = future
            .map(move |obj| sender.send(obj).unwrap())
            .map_err(|e| println!("Unexpected error while retrieving level for cache: {:?}", e));

        self.client.handle().spawn(future)
    }

    fn refresh_many<F>(&self, future: F)
        where
            F: Future<Item=Vec<GDObject>, Error=GDError> + 'static
    {
        let sender = self.sender.clone();
        let future = future
            .map(move |objs| {
                for obj in objs {
                    sender.send(obj).unwrap()
                }
            })
            .map_err(|e| println!("Unexpected error while retrieving level for cache: {:?}", e));

        self.client.handle().spawn(future)
    }
}