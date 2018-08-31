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
    E: Error + Send + 'static,
{
    fn fmt(&self, f: &mut Formatter) -> Result<(), fmt::Error> {
        match *self {
            CacheError::NoStore | CacheError::CacheMiss => write!(f, "{}", self.description()),
            CacheError::Custom(ref inner) => write!(f, "{}", inner),
        }
    }
}

impl<E> Display for ApiError<E>
where
    E: Error + Send + 'static,
{
    fn fmt(&self, f: &mut Formatter) -> Result<(), fmt::Error> {
        match *self {
            ApiError::NoData | ApiError::UnexpectedFormat | ApiError::InternalServerError => write!(f, "{}", self.description()),
            ApiError::MalformedData(ref inner) => write!(f, "Malformed response data: {}", inner),
            ApiError::Custom(ref inner) => write!(f, "{}", inner),
        }
    }
}

impl<A, C> Display for GdcfError<A, C>
where
    A: Error + Send + 'static,
    C: Error + Send + 'static,
{
    fn fmt(&self, f: &mut Formatter) -> Result<(), fmt::Error> {
        match *self {
            GdcfError::Cache(ref inner) => write!(f, "{}", inner),
            GdcfError::Api(ref inner) => write!(f, "{}", inner),
        }
    }
}

impl Error for ValueError {
    fn description(&self) -> &str {
        match *self {
            ValueError::IndexOutOfBounds(_) => "Index out of bounds",
            ValueError::NoValue(_) => "No value at specified index",
            ValueError::Parse(..) => "Failure to parse data at specified index",
        }
    }
}

impl<E> Error for CacheError<E>
where
    E: Error + Send + 'static,
{
    fn description(&self) -> &str {
        match *self {
            CacheError::CacheMiss => "The requested item isn't cached",
            CacheError::NoStore => "The cache refused to store the provided data",
            CacheError::Custom(ref inner) => inner.description(),
        }
    }
}

impl<E> Error for ApiError<E>
where
    E: Error + Send + 'static,
{
    fn description(&self) -> &str {
        match *self {
            ApiError::InternalServerError => "Internal server error",
            ApiError::NoData => "The response contained no data",
            ApiError::UnexpectedFormat => "The response format was unexpected",
            ApiError::MalformedData(ref inner) => inner.description(),
            ApiError::Custom(ref inner) => inner.description(),
        }
    }
}

impl<A, C> Error for GdcfError<A, C>
where
    A: Error + Send + 'static,
    C: Error + Send + 'static,
{
    fn description(&self) -> &str {
        match *self {
            GdcfError::Cache(ref inner) => inner.description(),
            GdcfError::Api(ref inner) => inner.description(),
        }
    }
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
    A: Error + Send + 'static,
    C: Error + Send + 'static,
{
    fn from(inner: ValueError) -> Self {
        GdcfError::Api(inner.into())
    }
}

impl<A, C> From<ApiError<A>> for GdcfError<A, C>
where
    A: Error + Send + 'static,
    C: Error + Send + 'static,
{
    fn from(inner: ApiError<A>) -> Self {
        GdcfError::Api(inner)
    }
}

impl<A, C> From<CacheError<C>> for GdcfError<A, C>
where
    A: Error + Send + 'static,
    C: Error + Send + 'static,
{
    fn from(inner: CacheError<C>) -> Self {
        GdcfError::Cache(inner)
    }
}
