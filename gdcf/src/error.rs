//! Module containing the various error types used by gdcf

use failure::Fail;

/// Type for errors that occur during cache access
#[derive(Debug, Fail)]
pub enum CacheError<E: Fail> {
    /// The cache chose not to store the provided value
    #[fail(display = "The cache refused to store the provided value")]
    NoStore,

    /// The value requested is not cached
    #[fail(display = "The requested value was not cached")]
    CacheMiss,

    /// An error caused by the underlying cache implementation occurred
    #[fail(display = "A cache-specific error occured: {}", _0)]
    Custom(#[cause] E),
}

/// Type for errors that occur when interacting with the API
#[derive(Debug, Fail)]
pub enum ApiError<E: Fail> {
    /// The API server returned a 500 INTERNAL SERVER ERROR response
    #[fail(display = "Internal Server Error")]
    InternalServerError,

    /// The request resulted in no data
    ///
    /// This can either be a 404 response, or an otherwise empty response, like
    /// RobTop's `-1` responses
    #[fail(display = "The request completed successfully, but no data was provided")]
    NoData,

    /// The response had an unexpected format
    #[fail(display = "Parsing of the response failed")]
    UnexpectedFormat,

    /// The request data was malformed
    ///
    /// This variant is only intended to be used while constructing data from
    /// [RawObject](model/structs.RawObject.html)s
    #[fail(display = "Processing the response data failed at index {}: {}", _0, _1)]
    MalformedData(String, String, #[cause] Box<dyn Fail>),

    #[fail(display = "Required data at index {} missing", _0)]
    MissingData(String),

    /// An error caused by the underlying api client implementation occured
    #[fail(display = "An API client specific error occurate: {}", _0)]
    Custom(#[cause] E),
}

#[derive(Debug, Fail)]
pub enum GdcfError<A: Fail, C: Fail> {
    #[fail(display = "{}", _0)]
    Cache(#[cause] CacheError<C>),

    #[fail(display = "{}", _0)]
    Api(#[cause] ApiError<A>),

    #[fail(display = "Neither cache-lookup, nor API response yielded any result")]
    NoContent,
}

impl<A: Fail, C: Fail> From<ApiError<A>> for GdcfError<A, C> {
    fn from(inner: ApiError<A>) -> Self {
        GdcfError::Api(inner)
    }
}

impl<A: Fail, C: Fail> From<CacheError<C>> for GdcfError<A, C> {
    fn from(inner: CacheError<C>) -> Self {
        GdcfError::Cache(inner)
    }
}
/*
impl<'a, F: Fail> From<ValueError<'a>> for ApiError<F> {
    fn from(inner: ValueError) -> Self {
        match inner {
            ValueError::NoValue(idx) => ApiError::MissingData(idx.to_owned()),
            ValueError::Parse(idx, value, err) => ApiError::MalformedData(idx.to_owned(), value.to_owned(), err),
        }
    }
}

impl<'a, A: Fail, C: Fail> From<ValueError<'a>> for GdcfError<A, C> {
    fn from(inner: ValueError) -> Self {
        GdcfError::Api(inner.into())
    }
}
*/
