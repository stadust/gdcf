//! Module containing traits/structs related to API calls
//!
//! Particularly, this contains all the structs modelling requests to the Geometry Dash API

pub mod client;
pub mod request;

pub use self::client::ApiClient;
