//! Module containing the various error types used by gdcf

use failure::Fail;

pub trait ApiError: Fail {
    fn is_no_result(&self) -> bool;
}

pub trait CacheError: Fail {}

#[derive(Debug, Fail)]
pub enum GdcfError<A: ApiError, C: CacheError> {
    #[fail(display = "{}", _0)]
    Cache(#[cause] C),

    #[fail(display = "{}", _0)]
    Api(#[cause] A),
}
