use cache::DatabaseCacheConfig;
use diesel::mysql::MysqlConnection;
use diesel::Connection;

pub struct DatabaseCache {
    pub(super) connection: MysqlConnection,
    pub(super) config: DatabaseCacheConfig,
}

pub fn connect(url: &str) -> MysqlConnection {
    MysqlConnection::establish(url).expect("Failed to connect to database")
}
