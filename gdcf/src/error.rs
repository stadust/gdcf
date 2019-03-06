//! Module containing the various error types used by gdcf

use failure::Fail;

pub trait ApiError: Fail {
    fn is_no_result(&self) -> bool;
}

pub trait CacheError: Fail {
    fn is_cache_miss(&self) -> bool;
}

#[derive(Debug, Fail)]
pub enum GdcfError<A: ApiError, C: CacheError> {
    #[fail(display = "{}", _0)]
    Cache(#[cause] C),

    #[fail(display = "{}", _0)]
    Api(#[cause] A),

    #[fail(display = "Neither cache-lookup, nor API response yielded any result")]
    NoContent,
}
