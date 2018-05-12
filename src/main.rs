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
use gdrs::BoomlingsClient;
use tokio_core::reactor::Core;

//use gdcf_dbcache::cache::{DatabaseCache, DatabaseCacheConfig};

fn main() {
    gdcf_dbcache::test();

    println!("{:?}", skyline(
        &vec![11, 0, 2, 5],
        &vec![2, 4, 4, 4],
        &vec![4, 4, 8, 2],
        0,
        4,
    ));
}


fn skyline(xi: &Vec<i32>, bi: &Vec<i32>, hi: &Vec<i32>, i: usize, n: usize) -> Vec<(i32, i32)> {
    use std::cmp::max;

    if n == 1 {
        vec![(xi[i], hi[i]), (xi[i] + bi[i], 0)]
    } else {
        let left = skyline(xi, bi, hi, i, n / 2);
        let right = skyline(xi, bi, hi, i + n / 2, n / 2);

        let mut result = vec![(0, 0); 2 * n];

        let mut lp = 0;
        let mut rp = 0;

        while lp + rp < 2 * n {
            if lp < n && (rp >= n || left[lp].0 < right[rp].0) {
                let (xl, hl) = left[lp];

                if rp == 0 {
                    result[lp + rp] = (xl, hl);
                } else {
                    result[lp + rp] = (xl, max(hl, right[rp - 1].1));
                }
                lp += 1;
            } else {
                let (xr, hr) = right[rp];

                if lp == 0 {
                    result[lp + rp] = (xr, hr);
                } else {
                    result[lp + rp] = (xr, max(hr, left[lp - 1].1));
                }
                rp += 1;
            }
        }

        result
    }
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
*/