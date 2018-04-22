use diesel::backend::Backend;
use diesel::connection::Connection;

use std::time::SystemTime;
use std::time::UNIX_EPOCH;
use gdcf::cache::CachedObject;
use chrono::NaiveDateTime;

#[macro_use]
mod macros;

pub mod song;


pub trait DBCached<Back: Backend, Prim>: Sized
{
    type Inner;

    fn get<Conn>(key: Prim, conn: &Conn) -> Option<CachedObject<Self::Inner>>
        where
            Conn: Connection<Backend=Back>;

    fn insert<Conn>(obj: Self::Inner, conn: &Conn)
        where
            Conn: Connection<Backend=Back>;
}