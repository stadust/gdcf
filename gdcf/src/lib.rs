#![feature(try_from)]
#![feature(box_syntax)]
#![feature(attr_literals)]
#![feature(never_type)]

#[macro_use]
#[cfg(ser)]
extern crate serde_derive;

#[cfg(ser)]
extern crate serde;

#[macro_use]
extern crate lazy_static;

extern crate futures;

extern crate tokio_core;

#[macro_use]
extern crate gdcf_derive;

use futures::Future;

use cache::Cache;
use model::{Level, PartialLevel};
use model::song::NewgroundsSong;
use model::ObjectType;
use model::FromRawObject;

use api::client::GDClient;
use api::request::level::LevelRequest;
use api::GDError;
use api::request::Request;

use std::sync::mpsc;
use std::sync::mpsc::Sender;
use std::thread;
use std::sync::Mutex;
use std::sync::Arc;
use std::sync::MutexGuard;
use model::RawObject;
use std::sync::mpsc::Receiver;
use api::request::LevelsRequest;

pub mod cache;
pub mod model;
pub mod api;

pub struct Gdcf<'a, A: 'a, C: 'static>
    where
        A: GDClient,
        C: Cache + Send
{
    cache: Arc<Mutex<C>>,
    client: &'a A,
    sender: Sender<RawObject>,
}

macro_rules! lookup {
    ($self: expr, $lookup: ident, $req: expr) => {
        {
            let cache = $self.cache();
            let cached = cache.$lookup(&$req);
            let expired = cached.as_ref()
                .map_or(true, |co| cache.is_expired(co));

            (cached, expired)
        }
    }
}

macro_rules! retrieve_one {
    ($name:ident, $req_type:tt, $lookup:ident, $api:tt) => {
        pub fn $name(&self, req: $req_type) -> Option<<$req_type as Request>::Result> {
            let (cached, expired) = lookup!(self, $lookup, req);

            if expired {
                self.refresh_one(self.client.$api(req));
            }

            cached.map(|co| co.extract())
        }
    }
}

macro_rules! retrieve_many {
    ($name:ident, $req_type:tt, $lookup:ident, $api:tt) => {
        pub fn $name(&self, req: $req_type) -> Option<<$req_type as Request>::Result> {
            let (cached, expired) = lookup!(self, $lookup, req);

            if expired {
                self.refresh_many(self.client.$api(req));
            }

            cached.map(|co| co.extract())
        }
    }
}

impl<'a, A: 'a, C: 'static> Gdcf<'a, A, C>
    where
        A: GDClient,
        C: Cache + Send
{
    pub fn new(cache: C, client: &A) -> Gdcf<A, C> {
        let (tx, rx): (Sender<RawObject>, Receiver<RawObject>) = mpsc::channel();
        let mutex = Arc::new(Mutex::new(cache));

        let handle = {
            let mutex = Arc::clone(&mutex);

            thread::spawn(move || {
                for raw_obj in rx {
                    let mut cache = mutex.lock().unwrap();

                    let err = match raw_obj.object_type {
                        ObjectType::Level => Level::from_raw(&raw_obj).map(|l| cache.store_level(l)),
                        ObjectType::PartialLevel => PartialLevel::from_raw(&raw_obj).map(|l| cache.store_partial_level(l)),
                        ObjectType::NewgroundsSong => NewgroundsSong::from_raw(&raw_obj).map(|s| cache.store_song(s))
                    };

                    if let Err(err) = err {
                        println!("Unexpected error while constructing object {:?}: {:?}", raw_obj.object_type, err)
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

    retrieve_one!(level, LevelRequest, lookup_level, level);
    retrieve_many!(levels, LevelsRequest, lookup_partial_levels, levels);

    fn refresh_one<F>(&self, future: F)
        where
            F: Future<Item=RawObject, Error=GDError> + 'static
    {
        let sender = self.sender.clone();
        let future = future
            .map(move |obj| sender.send(obj).unwrap())
            .map_err(|e| println!("Unexpected error while retrieving data for cache: {:?}", e));

        self.client.handle().spawn(future)
    }

    fn refresh_many<F>(&self, future: F)
        where
            F: Future<Item=Vec<RawObject>, Error=GDError> + 'static
    {
        let sender = self.sender.clone();
        let future = future
            .map(move |objs| objs.into_iter().for_each(|obj| sender.send(obj).unwrap()))
            .map_err(|e| println!("Unexpected error while retrieving data for cache: {:?}", e));

        self.client.handle().spawn(future)
    }
}