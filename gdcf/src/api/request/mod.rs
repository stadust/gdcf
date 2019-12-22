//! Module containing structs modelling the requests processable by the
//! Geometry Dash servers
//!
//! Each struct in this module is modelled strictly after the requests made by
//! the official Geometry Dash client.
//!
//! It further does not attempt to provide any (de)serialization for the
//! request types, as there are simply no sensible defaults. When providing
//! (de)serialization for requests, take a look at solutions like serde's
//! remote types.

pub use self::{
    level::{LevelRequest, LevelRequestType, LevelsRequest, SearchFilters, SongFilter},
    user::{UserRequest, UserSearchRequest},
    comment::{ProfileCommentsRequest, LevelCommentsRequest}
};
use gdcf_model::GameVersion;
use std::{fmt::Debug, hash::Hash};

pub mod comment;
pub mod level;
pub mod user;

/// A `BaseRequest` instance that has all its fields set to the
/// same values a Geometry Dash 2.1 client would use
pub const GD_21: BaseRequest = BaseRequest::new(
    GameVersion::Version { major: 2, minor: 1 },
    GameVersion::Version { major: 3, minor: 3 },
    "Wmfd2893gb7",
);

/// Base data included in every request made
///
/// The fields in this struct are only relevant when making a request to the
/// `boomlings` servers. When using GDCF with a custom Geometry Dash API, they
/// can safely be ignored.
#[derive(Debug, Clone, Hash, Copy, PartialEq, Eq)]
pub struct BaseRequest {
    /// The version of the game client we're pretending to be
    ///
    /// ## GD Internals:
    /// This field is called `gameVersion` in the boomlings API and needs to be
    /// converted to a string response
    /// The value of this field doesn't matter, and the request will succeed
    /// regardless of what it's been set to
    pub game_version: GameVersion,

    /// Internal version of the game client we're pretending to be
    ///
    /// ## GD Internals:
    /// This field is called `binaryVersion` in the boomlings API and needs to
    /// be converted to a string
    ///
    /// The value of this field doesn't matter, and the request will succeed
    /// regardless of what it's been set to
    pub binary_version: GameVersion,

    /// The current secret String the server uses to identify valid clients.
    ///
    /// ## GD Internals:
    /// Settings this field to an incorrect value will cause the request to fail
    pub secret: &'static str,
}

impl BaseRequest {
    /// Constructs a new `BaseRequest` with the given values.
    pub const fn new(game_version: GameVersion, binary_version: GameVersion, secret: &'static str) -> BaseRequest {
        BaseRequest {
            game_version,
            binary_version,
            secret,
        }
    }
}

impl Default for BaseRequest {
    fn default() -> Self {
        GD_21
    }
}

/// Trait for types that are meant to be requests whose results can be cached
/// by GDCF.
pub trait Request: Debug + Send + Sync + 'static {
    /// The type of object returned by this request.
    ///
    /// For requests that return multiple types of objects (like [`LevelsRequest`], which returns
    /// levels, songs and creators), this is the non-[`Secondary`] object returned by this request
    /// (so the vector of [`PartialLevel`]s in the above example) .
    type Result: Debug + Send + Sync + 'static;
}

/// Trait for requests that can be seen as returning pages of objects.
///
/// In general, these are requests like [`LevelsRequest`], which returns pages of levels. However,
/// also requests like [`LevelRequest`] can be seen as paginatable (and does in fact implement this
/// trait) because we can interpret a level with some level ID `n` to be the `n-`th page of the
/// request.
pub trait PaginatableRequest: Request {
    /// Modifies this request in-place to be a request for the next page
    fn next(&mut self);
}
