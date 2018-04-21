use diesel::sqlite::SqliteConnection;
use diesel::Connection;

pub struct DatabaseCache {
    pub(super) connection: SqliteConnection
}

pub fn connect(url: &str) -> SqliteConnection {
    SqliteConnection::establish(url).expect("Failed to connect to database")
}