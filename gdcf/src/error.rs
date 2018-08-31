//! Module containing the various error types used by gdcf

use std::{
    error::Error,
    fmt::{self, Display, Formatter},
};

#[derive(Debug)]
pub enum ValueError {
    IndexOutOfBounds(usize),
    NoValue(usize),
    Parse(usize, String, Box<dyn Error + Send + 'static>),
}

/// Type for errors that occur during cache access
#[derive(Debug)]
pub enum CacheError<E>
where
    E: Error + Send + 'static,
{
    /// The cache chose not to store the provided value
    NoStore,

    /// The value requested is not cached
    CacheMiss,

    /// An error caused by the underlying cache implementation occurred
    Custom(E),
}

/// Type for errors that occur when interacting with the API
#[derive(Debug)]
pub enum ApiError<E>
where
    E: Error + Send + 'static,
{
    /// The API server returned a 500 INTERNAL SERVER ERROR response
    InternalServerError,

    /// The request resulted in no data
    ///
    /// This can either be a 404 response, or an otherwise empty response, like
    /// RobTop's `-1` responses
    NoData,

    /// The response had an unexpected format
    UnexpectedFormat,

    /// The request data was malformed
    ///
    /// This variant is only intended to be used while constructing data from
    /// [RawObject](model/structs.RawObject.html)s
    MalformedData(ValueError),

    /// An error caused by the underlying api client implementation occured
    Custom(E),
}

#[derive(Debug)]
pub enum GdcfError<A, C>
where
    A: Error + Send + 'static,
    C: Error + Send + 'static,
{
    Cache(CacheError<C>),
    Api(ApiError<A>),

    NoContent,
}

impl Display for ValueError {
    fn fmt(&self, f: &mut Formatter) -> Result<(), fmt::Error> {
        match *self {
            ValueError::IndexOutOfBounds(idx) => write!(f, "Index {} was out of bounds", idx),
            ValueError::NoValue(idx) => write!(f, "No value provided at index {}", idx),
            ValueError::Parse(idx, ref string, ref err) => write!(f, "Failed to parse value at index {} ('{}'): {}", idx, string, err),
        }
    }
}

impl<E> Display for CacheError<E>
where
    E: Error + Send,
{
    fn fmt(&self, f: &mut Formatter) -> Result<(), fmt::Error> {
        match *self {
            CacheError::CacheMiss => write!(f, "The requested item isn't cached"),
            CacheError::NoStore => write!(f, "The cache refused to store the provided data"),
            CacheError::Custom(ref inner) => write!(f, "{}", inner),
        }
    }
}

impl<E> Display for ApiError<E>
where
    E: Error + Send,
{
    fn fmt(&self, f: &mut Formatter) -> Result<(), fmt::Error> {
        match *self {
            ApiError::InternalServerError => write!(f, "Internal server error"),
            ApiError::NoData => write!(f, "The response contained no data"),
            ApiError::UnexpectedFormat => write!(f, "The response format was unexpected"),
            ApiError::MalformedData(ref inner) => write!(f, "Malformed response data: {}", inner),
            ApiError::Custom(ref inner) => write!(f, "{}", inner),
        }
    }
}

impl<A, C> Display for GdcfError<A, C>
where
    A: Error + Send,
    C: Error + Send,
{
    fn fmt(&self, f: &mut Formatter) -> Result<(), fmt::Error> {
        match *self {
            GdcfError::Cache(ref inner) => write!(f, "{}", inner),
            GdcfError::Api(ref inner) => write!(f, "{}", inner),
            GdcfError::NoContent => write!(f, "Request was successful, yet yielded no actual data"),
        }
    }
}

impl Error for ValueError {}

impl<E> Error for CacheError<E> where E: Error + Send {}

impl<E> Error for ApiError<E> where E: Error + Send {}

impl<A, C> Error for GdcfError<A, C>
where
    A: Error + Send,
    C: Error + Send,
{
}

impl<E> From<ValueError> for ApiError<E>
where
    E: Error + Send + 'static,
{
    fn from(inner: ValueError) -> Self {
        ApiError::MalformedData(inner)
    }
}

impl<A, C> From<ValueError> for GdcfError<A, C>
where
    A: Error + Send,
    C: Error + Send,
{
    fn from(inner: ValueError) -> Self {
        GdcfError::Api(inner.into())
    }
}

impl<A, C> From<ApiError<A>> for GdcfError<A, C>
where
    A: Error + Send,
    C: Error + Send,
{
    fn from(inner: ApiError<A>) -> Self {
        GdcfError::Api(inner)
    }
}

impl<A, C> From<CacheError<C>> for GdcfError<A, C>
where
    A: Error + Send,
    C: Error + Send,
{
    fn from(inner: CacheError<C>) -> Self {
        GdcfError::Cache(inner)
    }
}
