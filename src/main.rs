extern crate chrono;
extern crate env_logger;
extern crate futures;
extern crate gdcf;
extern crate gdcf_dbcache;
extern crate gdrs;
#[macro_use]
extern crate log;
extern crate tokio;

use chrono::Duration;
use futures::{Async, Future, lazy};
use gdcf::api::request::{LevelRequest, LevelsRequest, Request};
use gdcf::api::request::LevelRequestType;
use gdcf::cache::Cache;
use gdcf::Gdcf;
use gdcf_dbcache::cache::{DatabaseCache, DatabaseCacheConfig};
use gdrs::BoomlingsClient;
use tokio::executor::current_thread;

fn main() {
    env_logger::init();

    let config = DatabaseCacheConfig::postgres_config("postgres://gdcf:gdcf@localhost/gdcf");
    let cache = DatabaseCache::new(config);

    cache.initialize().expect("Error initializing cache");

    let client = BoomlingsClient::new();
    let gdcf = Gdcf::new(client, cache);

    tokio::run(lazy(move || {
        let request = LevelsRequest::default()
            .request_type(LevelRequestType::Featured)
            .page(5);

        gdcf.levels(request)
            .map_err(|error| eprintln!("Error retrieving 5th page of featured levels!"))
            .map(move |levels| {
                for level in levels {
                    let future = gdcf.level(LevelRequest::new(level.level_id))
                        .map(|l| println!("Password of level {}: {:?}", l, l.password))
                        .map_err(move |error| eprintln!("Error downloading level {}: {:?}", level.level_id, error));

                    tokio::spawn(future);
                }
            })
    }));
}