extern crate gdrs;
extern crate gdcf;

extern crate tokio_core;
extern crate futures;
extern crate serde_urlencoded;
extern crate serde_json;
extern crate gdcf_dbcache;
extern crate chrono;

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
use gdcf::model::song::NewgroundsSong;
use gdcf::api::request::LevelsRequest;
use gdcf::api::request::LevelRequest;
use gdcf::api::request::level::SearchFilters;

use gdcf_dbcache::cache::DatabaseCache;
use gdcf_dbcache::cache::DatabaseCacheConfig;

use chrono::Duration;

fn main() {
    let mut core = Core::new().unwrap();
    let client = GDClientImpl::new(&core.handle());
    let config = DatabaseCacheConfig::new("postgres://gdcf:gdcf@localhost/gdcf", Duration::seconds(0));
    let cache = DatabaseCache::new(config);

    let gdcf = Gdcf::new(cache, &client);

    let lev_req = LevelsRequest::default()
        .search("Under Lavaland".into())
        .filter(SearchFilters::default()
            .featured()
            .uncompleted());

    gdcf.levels(lev_req);
    gdcf.level(11774780.into());
    gdcf.level(11849346.into());

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