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

    #[fail(display = "GDCF made an assumption about server sided data consistency, which was violated. Please open a bug report")]
    ConsistencyAssumptionViolated,

    #[fail(display = "The response produced by the APi client implementation didn't contain the expected data")]
    ResponseMismatch,
}
