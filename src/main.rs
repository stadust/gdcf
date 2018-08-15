#![deny(
bare_trait_objects, missing_debug_implementations, unused_extern_crates, patterns_in_fns_without_body, stable_features, unknown_lints, unused_features, unused_imports, unused_parens
)]

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
use gdcf::Gdcf;
use gdcf_dbcache::cache::{DatabaseCache, DatabaseCacheConfig};
use gdrs::BoomlingsClient;

fn main() {
    env_logger::init();

    let mut config = DatabaseCacheConfig::postgres_config("postgres://gdcf:gdcf@localhost/gdcf");
    config.invalidate_after(Duration::minutes(30));
    let cache = DatabaseCache::new(config);

    let d = Duration::minutes(30);

    cache.initialize().expect("Error initializing cache");

    let client = BoomlingsClient::new();
    let gdcf = Gdcf::new(client, cache);

    tokio::run(lazy(move || {
        let request = LevelsRequest::default()
            .request_type(LevelRequestType::Featured)
            .page(1);

        gdcf.levels(request)
            .map_err(|_| eprintln!("Error retrieving 6th page of featured levels!"))
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