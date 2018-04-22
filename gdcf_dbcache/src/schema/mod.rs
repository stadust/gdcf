use diesel::backend::Backend;
use diesel::connection::Connection;

use chrono::NaiveDateTime;
use gdcf::cache::CachedObject;
use std::time::SystemTime;
use std::time::UNIX_EPOCH;

#[macro_use]
mod macros;

pub mod song;

pub trait DBCached<Back: Backend>: Sized {
    type Inner;
    type SearchKey;

    fn get<Conn>(key: Self::SearchKey, conn: &Conn) -> Option<CachedObject<Self::Inner>>
    where
        Conn: Connection<Backend = Back>;

    fn insert<Conn>(obj: Self::Inner, conn: &Conn)
    where
        Conn: Connection<Backend = Back>;
}
