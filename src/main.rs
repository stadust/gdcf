extern crate gdcf;
extern crate gdrs;

extern crate chrono;
extern crate futures;
extern crate gdcf_dbcache;
extern crate serde_json;
extern crate serde_urlencoded;
extern crate tokio_core;
extern crate env_logger;

use chrono::Duration;

use futures::{Async, Future};

use tokio_core::reactor::Core;

use gdcf::api::request::{Request, LevelRequest, LevelsRequest};
use gdcf::{Gdcf, ConsistentCacheManager};

use gdcf_dbcache::cache::{DatabaseCache, DatabaseCacheConfig};

use gdrs::BoomlingsClient;

fn main() {
    env_logger::init();

    let mut core = Core::new().unwrap();

    let client = BoomlingsClient::new(&core.handle());
    let config = DatabaseCacheConfig::new("postgres://gdcf:gdcf@localhost/gdcf", Duration::seconds(0));
    let cache = DatabaseCache::new(config);

    let gdcf = ConsistentCacheManager::new(client, cache);

    let levels = vec![38786978u64, 38515466u64, 11774780u64, 39599737u64, 1u64];

    for level in levels.into_iter() {
        gdcf.level(LevelRequest::new(level));
    }

    core.run(until_all_done());
}

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
