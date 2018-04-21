use diesel::backend::Backend;
use diesel::connection::Connection;

use std::time::SystemTime;
use std::time::UNIX_EPOCH;

#[macro_use]
mod macros;

pub mod song;


pub trait Cached<Back: Backend, Prim>: Sized
{
    fn get<Conn>(key: Prim, conn: &Conn) -> Option<Self>
        where
            Conn: Connection<Backend=Back>;

    fn insert<Conn>(self, conn: &Conn)
        where
            Conn: Connection<Backend=Back>;

    fn replace_with<Conn>(self, new: Self, conn: &Conn)
        where
            Conn: Connection<Backend=Back>;
}

pub fn now() -> u64 {
    SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs()
}