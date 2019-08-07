use failure_derive::Fail;
use gdcf::error::ApiError as TApiError;
use gdcf_parse::error::ValueError;
use tokio_retry::Error as RetryError;

#[derive(Fail, Debug)]
pub enum ApiError {
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
    #[fail(
        display = "Processing the response data failed at index {} due to value '{}': {}",
        index, value, msg
    )]
    MalformedData { index: String, value: String, msg: String },

    #[fail(display = "Required data at index {} missing", _0)]
    MissingData(String),

    /// An error caused by the underlying api client implementation occured
    #[fail(display = "An API client specific error occurate: {}", _0)]
    Custom(#[cause] hyper::Error),
}

impl<'a> From<ValueError<'a>> for ApiError {
    fn from(inner: ValueError) -> Self {
        match inner {
            ValueError::NoValue(idx) => ApiError::MissingData(idx.to_owned()),
            ValueError::Parse(idx, value, err) =>
                ApiError::MalformedData {
                    index: idx.to_owned(),
                    value: value.to_owned(),
                    msg: err,
                },
        }
    }
}

impl TApiError for ApiError {
    fn is_no_result(&self) -> bool {
        match self {
            ApiError::InternalServerError | ApiError::NoData => true,
            _ => false,
        }
    }
}

impl From<RetryError<ApiError>> for ApiError {
    fn from(err: RetryError<ApiError>) -> Self {
        match err {
            RetryError::OperationError(e) => e,
            _ => unimplemented!(),
        }
    }
}
