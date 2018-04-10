extern crate gdrs;
extern crate gdcf;

extern crate tokio_core;
extern crate futures;
extern crate serde_urlencoded;

use gdcf::cache::Cache;
use gdcf::cache::CacheConfig;
use gdcf::cache::CachedObject;
use gdcf::model::PartialLevel;
use gdcf::model::Level;

use gdrs::GDClientImpl;

use tokio_core::reactor::Core;
use gdcf::Gdcf;
use futures::Future;
use futures::Async;

fn main() {
    let mut core = Core::new().unwrap();
    let client = GDClientImpl::new(&core.handle());
    let cache = DummyCache {};

    let gdcf = Gdcf::new(cache, client);

    gdcf.level(39976494);
    gdcf.level(1);
    gdcf.level(3150);

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

struct DummyCache;

impl Cache for DummyCache {
    fn config(&self) -> CacheConfig {
        CacheConfig {
            invalidate_after: 0
        }
    }

    fn lookup_partial_level(&self, lid: u64) -> Option<CachedObject<PartialLevel>> {
        None
    }

    fn store_partial_level(&mut self, level: PartialLevel) {

    }

    fn lookup_level(&self, lid: u64) -> Option<CachedObject<Level>> {
        None
    }

    fn store_level(&mut self, level: Level) {
        println!("{:?}", level);
    }
}