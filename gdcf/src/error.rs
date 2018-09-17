//! Module containing the various error types used by gdcf

use failure::Fail;

#[derive(Debug, Fail)]
pub enum ValueError {
    #[fail(display = "No value provided at index {}", _0)]
    NoValue(usize),

    #[fail(display = "The value '{}' at index {} could not be parsed: {}", _1, _0, _2)]
    Parse(usize, String, #[cause] Box<dyn Fail>),
}

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
    #[fail(display = "Processing the response data failed: {}", _0)]
    MalformedData(#[cause] ValueError),

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

impl<E: Fail> From<ValueError> for ApiError<E> {
    fn from(inner: ValueError) -> Self {
        ApiError::MalformedData(inner)
    }
}

impl<A: Fail, C: Fail> From<ValueError> for GdcfError<A, C> {
    fn from(inner: ValueError) -> Self {
        GdcfError::Api(inner.into())
    }
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
