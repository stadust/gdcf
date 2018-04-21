extern crate gdrs;
extern crate gdcf;

extern crate tokio_core;
extern crate futures;
extern crate serde_urlencoded;
extern crate serde_json;
extern crate gdcf_dbcache;

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

fn main() {
    let mut core = Core::new().unwrap();
    let client = GDClientImpl::new(&core.handle());
    let cache = DatabaseCache::new("postgres://gdcf:gdcf@localhost/gdcf");

    let gdcf = Gdcf::new(cache, &client);

    let lev_req = LevelsRequest::default()
        .search("Silent Club".into())
        .filter(SearchFilters::default()
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