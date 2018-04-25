extern crate gdcf;
extern crate gdrs;

extern crate chrono;
extern crate futures;
extern crate gdcf_dbcache;
extern crate serde_json;
extern crate serde_urlencoded;
extern crate tokio_core;
extern crate env_logger;

use gdrs::BoomlingsClient;

use chrono::Duration;
use futures::Async;
use futures::Future;
use gdcf::api::request::level::SearchFilters;
use gdcf::api::request::LevelsRequest;
use gdcf::Gdcf;
use gdcf_dbcache::cache::DatabaseCache;
use gdcf_dbcache::cache::DatabaseCacheConfig;
use tokio_core::reactor::Core;
use gdcf::CacheManager;
use gdcf::ConsistentCacheManager;

fn main() {
    env_logger::init();

    let mut core = Core::new().unwrap();

    let client = BoomlingsClient::new(&core.handle());
    let config = DatabaseCacheConfig::new("postgres://gdcf:gdcf@localhost/gdcf", Duration::seconds(0));
    let cache = DatabaseCache::new(config);

    let gdcf = ConsistentCacheManager::new(client, cache);

    for level in vec![44802267u64, 43120057, 9] {
        gdcf.level(level.into());
    }

    core.run(Thing {});
}

struct Thing;

impl Future for Thing {
    type Item = ();
    type Error = ();

    fn poll(&mut self) -> Result<Async<()>, ()> {
        Ok(Async::NotReady)
    }
}
