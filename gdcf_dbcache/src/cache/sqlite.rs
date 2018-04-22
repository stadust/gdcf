use diesel::sqlite::SqliteConnection;
use diesel::Connection;
use cache::DatabaseCacheConfig;

pub struct DatabaseCache {
    pub(super) connection: SqliteConnection,
    pub(super) config: DatabaseCacheConfig,
}

pub fn connect(url: &str) -> SqliteConnection {
    SqliteConnection::establish(url).expect("Failed to connect to database")
}