//! Module containing structs modelling the requests processable by the Geometry Dash servers
//!
//! Each struct in this module is modelled strictly after the requests made by the official
//! Geometry Dash client.
//!
//! It further does not attempt to provide any (de)serialization for the request types,
//! as there are simply no sensible defaults. When providing (de)serialization for requests,
//! take a look at solutions like serde's remote types.
//!
//! # Creating custom request structs
//!
//! Libraries extending GDCF can define custom requests by implementing the [Request](trait.Request.html)
//! trait. The [gdcf!](../../macro.gdcf.html) macro can then be used to generate the boilerplate code that
//! links the request to a cache lookup, allowing it to be used with an implementation of the
//! [Gdcf](../../trait.Gdcf.html) trait.

use api::client::{ApiClient, ApiFuture};
use model::GameVersion;
pub use self::level::{LevelRequest, LevelRequestType, LevelsRequest, SearchFilters, SongFilter};
use std::fmt::Display;
use api::request::cacher::Cacher;
use std::hash::Hash;

pub mod level;
pub mod cacher;

/// Base data included in every request made
///
/// The fields in this struct are only relevant when making a request to the `boomlings` servers.
/// When using GDCF with a custom Geometry Dash API, they can safely be ignored.
#[derive(Debug, Clone, Hash)]
pub struct BaseRequest {
    /// The version of the game client we're pretending to be
    ///
    /// ## GD Internals:
    /// This field is called `gameVersion` in the boomlings API and needs to be converted to a string
    ///response
    /// The value of this field doesn't matter, and the request will succeed regardless of
    /// what it's been set to
    pub game_version: GameVersion,

    /// Internal version of the game client we're pretending to be
    ///
    /// ## GD Internals:
    /// This field is called `binaryVersion` in the boomlings API and needs to be converted to a string
    ///
    /// The value of this field doesn't matter, and the request will succeed regardless of
    /// what it's been set to
    pub binary_version: GameVersion,

    /// The current secret String the server uses to identify valid clients.
    ///
    /// ## GD Internals:
    /// Settings this field to an incorrect value will cause the request to fail
    pub secret: String,
}

impl BaseRequest {
    /// Constructs a new `BaseRequest` with the given values.
    pub fn new(game_version: GameVersion, binary_version: GameVersion, secret: String) -> BaseRequest {
        BaseRequest {
            game_version,
            binary_version,
            secret,
        }
    }

    /// Constructs a `BaseRequest` instance that has all its fields set to the same
    /// values a Geometry Dash 2.1 client would use
    pub fn gd_21() -> BaseRequest {
        BaseRequest::new(
            GameVersion::Version { major: 2, minor: 1 },
            GameVersion::Version { major: 3, minor: 3 },
            "Wmfd2893gb7".into(),
        )
    }
}

impl Default for BaseRequest {
    fn default() -> Self {
        BaseRequest::gd_21()
    }
}

/// Trait for types that are meant to be requests whose results can be cached by GDCF
///
/// A `Request`'s `Hash` result must be forward-compatible with new fields added to a request.
/// This means that if the GD API adds a new fields to a requests, making a request without this
/// fields should generate the same hash value as the same request in an old version of GDCF without
/// the field in the first place.
/// This means foremost, that `Hash` impls mustn't hash the `BaseRequest` they're built upon.
/// If new fields are added in later version of GDCF, they may only be hashed if they are explicitly
/// set to a value, to ensure the above-mentioned compatibility
pub trait Request: Display + Default + Hash {
    type ResponseCacher: Cacher<Self>;

    /// Creates a default instance of this `Request`
    fn new() -> Self {
        Default::default()
    }

    /// Makes this `Request` through the given `ApiClient`
    ///
    /// This method pretty much just exists so that GDCF can use static dispatch for
    /// request types internally. You should never have to call this manually and instead
    /// should use the methods provided by your implementation of `ApiClient`
    fn make<C: ApiClient>(&self, client: &C) -> ApiFuture<C::Err>;
}