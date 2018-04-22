use diesel::pg::PgConnection;
use diesel::Connection;

use cache::DatabaseCacheConfig;

pub struct DatabaseCache {
    pub(super) connection: PgConnection,
    pub(super) config: DatabaseCacheConfig
}

pub fn connect(url: &str) -> PgConnection {
    PgConnection::establish(url).expect("Failed to connect to database")
}