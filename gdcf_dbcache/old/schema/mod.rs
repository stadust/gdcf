use chrono::NaiveDateTime;
use diesel::backend::Backend;
use diesel::connection::Connection;
use diesel::deserialize::FromSql;
use diesel::result::Error;
use gdcf::cache::CachedObject;
use gdcf::error::CacheError;

#[macro_use]
mod macros;

pub mod song;
pub mod level;

pub trait Cached<Back: Backend>: Sized {
    type SearchKey;
    type Inner;

    fn retrieve<Conn>(key: Self::SearchKey, conn: &Conn) -> Result<Self, Error>
        where
            Conn: Connection<Backend=Back>;

    fn store<Conn>(obj: Self::Inner, conn: &Conn) -> Result<(), Error>
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
