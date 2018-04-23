pub mod client;
pub mod request;

pub use self::client::ApiClient;

use model::ValueError;

use std::str::Utf8Error;

#[derive(Debug)]
pub enum GDError {
    InternalServerError,
    ServersDown,
    Timeout,

    NoData,

    MalformedResponse,
    Value(ValueError),
    Encoding(Utf8Error),

    Unspecified,
}

impl From<Utf8Error> for GDError {
    fn from(err: Utf8Error) -> Self {
        GDError::Encoding(err)
    }
}

impl From<ValueError> for GDError {
    fn from(err: ValueError) -> Self {
        GDError::Value(err)
    }
}
