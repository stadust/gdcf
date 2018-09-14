//#![deny(
//bare_trait_objects, missing_debug_implementations, unused_extern_crates,
// patterns_in_fns_without_body, stable_features, unknown_lints,
// unused_features, unused_imports, unused_parens
//)]

extern crate chrono;
extern crate env_logger;
extern crate futures;
extern crate gdcf;
extern crate gdcf_dbcache;
extern crate gdrs;
extern crate tokio;

use chrono::Duration;
use futures::{lazy, stream::Stream, Future};
use gdcf::{
    api::request::{LevelRequest, LevelRequestType, LevelsRequest, SearchFilters},
    model::{DemonRating, LevelRating, NewgroundsSong},
    Gdcf,
};
use gdcf_dbcache::cache::{DatabaseCache, DatabaseCacheConfig};
use gdrs::BoomlingsClient;
use std::io::{self, Read};
use gdcf::model::Creator;

fn main() {
    env_logger::init();

    let mut config = DatabaseCacheConfig::postgres_config("postgres://gdcf:gdcf@localhost/gdcf");
    //let mut config = DatabaseCacheConfig::sqlite_config("/home/patrick/gd.sqlite");


    //let mut config = DatabaseCacheConfig::sqlite_memory_config();
    config.invalidate_after(Duration::minutes(3000));

    let cache = DatabaseCache::new(config);

    cache.initialize().expect("Error initializing cache");

    let client = BoomlingsClient::new();
    let gdcf = Gdcf::new(client, cache);

    tokio::run(lazy(move || {
        /*let request = LevelsRequest::default()
            .request_type(LevelRequestType::Featured);
            //.with_rating(LevelRating::Demon(DemonRating::Hard));

        gdcf.paginate_levels::<NewgroundsSong, Creator>(request)
            .take(50)
            .for_each(|levels| {
                print!("We got {} levels: ", levels.len());

                for level in levels {
                    print!("{} with song {:?} by {:?} ", level, level.custom_song, level.creator)
                }

                Ok(println!())
            }).map_err(|err| eprintln!("Something went wrong /shrug: {:?}", err))*/
        gdcf.user(8451.into())
            .map_err(|err| eprintln!("Something went wrong /shrug: {:?}", err))
            .map(|_|())
    }));

    println!("Press return to continue...");

    let _ = io::stdin().read(&mut [0u8]).unwrap();
}
