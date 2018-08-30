//#![deny(
//bare_trait_objects, missing_debug_implementations, unused_extern_crates, patterns_in_fns_without_body, stable_features, unknown_lints, unused_features, unused_imports, unused_parens
//)]

extern crate chrono;
extern crate env_logger;
extern crate futures;
extern crate gdcf;
extern crate gdcf_dbcache;
extern crate gdrs;
extern crate tokio;

use chrono::Duration;
use futures::{Future, lazy};
use gdcf::api::request::{LevelRequest, LevelsRequest};
use gdcf::api::request::LevelRequestType;
use gdcf::api::request::SearchFilters;
use gdcf::Gdcf;
use gdcf_dbcache::cache::{DatabaseCache, DatabaseCacheConfig};
use gdrs::BoomlingsClient;
use std::io;
use std::io::Read;
use gdcf::model::LevelRating;
use gdcf::model::DemonRating;

fn main() {
    env_logger::init();

    let mut config = DatabaseCacheConfig::postgres_config("postgres://gdcf:gdcf@localhost/gdcf");
    //let mut config = DatabaseCacheConfig::sqlite_memory_config();
    config.invalidate_after(Duration::minutes(30));
    let cache = DatabaseCache::new(config);

    cache.initialize().expect("Error initializing cache");

    let client = BoomlingsClient::new();
    let gdcf = Gdcf::new(client, cache);

    /*tokio::run(lazy(move || {
        let request = LevelsRequest::default()
            .request_type(LevelRequestType::Featured)
            .page(89);

        gdcf.levels(request)
            .map_err(|err| eprintln!("Error retrieving 6th page of featured levels! {:?}", err))
            .map(move |levels| {
                for level in levels {
                    let future = gdcf.level(LevelRequest::new(level.level_id))
                        .map(|l| println!("Password of level {}: {:?}", l, l.password))
                        .map_err(move |error| eprintln!("Error downloading level {}: {:?}", level.level_id, error));

                    tokio::spawn(future);
                }
            })
    }));*/
    tokio::run(lazy(move || {
        let request = LevelsRequest::default()
            .search("Dark Realm".to_string())
            .filter(SearchFilters::default()
                .rated())
            .with_rating(LevelRating::Demon(DemonRating::Hard));

        gdcf.levels(request)
            .map_err(|err| eprintln!("Error retrieving levels! {:?}", err))
            .map(move |mut levels| {
                let first = levels.remove(0);

                let future = gdcf.level(first.level_id.into())
                    .map_err(move |error| eprintln!("Error downloading level {}({}): {:?}", first.name, first.level_id, error))
                    .map(|level| println!("The password of the level {}({}) is: {:?}", level.base.name, level.base.level_id, level.password));

                tokio::spawn(future);
            })
    }));

    println!("Press return to continue...");

    let _ = io::stdin().read(&mut [0u8]).unwrap();
}