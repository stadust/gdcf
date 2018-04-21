use diesel::pg::PgConnection;
use diesel::Connection;

pub struct DatabaseCache {
    pub(super) connection: PgConnection
}

pub fn connect(url: &str) -> PgConnection {
    PgConnection::establish(url).expect("Failed to connect to database")
}