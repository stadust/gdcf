extern crate chrono;
extern crate env_logger;
extern crate futures;
extern crate gdcf;
extern crate gdcf_dbcache;
extern crate gdrs;
extern crate serde_json;
extern crate serde_urlencoded;
extern crate tokio_core;

use chrono::Duration;
use futures::{Async, Future};
use gdcf::{ConsistentCacheManager, Gdcf};
use gdcf::api::request::{LevelRequest, LevelsRequest, Request};
use gdcf_dbcache::cache::{DatabaseCache, DatabaseCacheConfig};
use gdrs::BoomlingsClient;
use tokio_core::reactor::Core;

//use gdcf_dbcache::cache::{DatabaseCache, DatabaseCacheConfig};

fn main() {
    env_logger::init();

    // Rust built-in await/async WHEN
    let mut core = Core::new().unwrap();

    let config = DatabaseCacheConfig::postgres_config("postgres://gdcf:gdcf@localhost/gdcf");
    let cache = DatabaseCache::new(config);

    gdcf_dbcache::test();
    cache.initialize().expect("Error initializing cache");

    let client = BoomlingsClient::new(&core.handle());

    let gdcf = ConsistentCacheManager::new(client, cache);


    gdcf.level(38515466u64.into());
    gdcf.levels(LevelsRequest::new().search("Auto play area".to_string()));



    core.run(until_all_done());

    /*println!("{:?}", skyline(
        &vec![11, 0, 2, 5],
        &vec![2, 4, 4, 4],
        &vec![4, 4, 8, 2],
        0,
        4,
    ));*/
}
/*fn main() {
    env_logger::init();

    let mut core = Core::new().unwrap();

    let client = BoomlingsClient::new(&core.handle());
    let config = DatabaseCacheConfig::new("postgres://gdcf:gdcf@localhost/gdcf", Duration::seconds(0));
    let cache = DatabaseCache::new(config);

    let gdcf = ConsistentCacheManager::new(client, cache);

    let levels = vec![38786978u64, 38515466u64, 11774780u64, 39599737u64, 3150u64];

    for level in levels.into_iter() {
        gdcf.level(LevelRequest::new(level));
    }

    gdcf.levels(LevelsRequest::new().search("Auto play area".to_string()));



    core.run(until_all_done());
}*/

pub fn until_all_done() -> impl Future<Item=(), Error=()> {
    Thing {}
}

struct Thing;

impl Future for Thing {
    type Item = ();
    type Error = ();

    fn poll(&mut self) -> Result<Async<()>, ()> {
        Ok(Async::NotReady)
    }
}
