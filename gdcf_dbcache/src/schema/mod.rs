use diesel::backend::Backend;
use diesel::connection::Connection;

use chrono::NaiveDateTime;
use gdcf::cache::CachedObject;
use std::time::SystemTime;
use std::time::UNIX_EPOCH;

#[macro_use]
mod macros;

pub mod song;

pub trait Cached<Back: Backend>: Sized {
    type SearchKey;
    type Inner;

    fn retrieve<Conn>(key: Self::SearchKey, conn: &Conn) -> Option<Self>
        where
            Conn: Connection<Backend=Back>;

    fn store<Conn>(obj: Self::Inner, conn: &Conn)
        where
            Conn: Connection<Backend=Back>;
}

/// Type to circumvent the orphan rules
struct _O<T> {
    c: CachedObject<T>
}

impl<T> _O<T> {
    fn new(c: CachedObject<T>) -> _O<T> {
        _O { c }
    }
}

impl<T> Into<CachedObject<T>> for _O<T> {
    fn into(self) -> CachedObject<T> {
        self.c
    }
}

impl<T> From<CachedObject<T>> for _O<T> {
    fn from(obj: CachedObject<T>) -> Self {
        _O::new(obj)
    }
}