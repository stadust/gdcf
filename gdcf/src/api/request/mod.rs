pub mod level;

pub use self::level::{LevelRequest, LevelsRequest, SearchFilters, LevelRequestType, SongFilter};
use model::GameVersion;

use api::ApiClient;
use api::client::ApiFuture;

use std::fmt::Display;

/// Base data included in every request made
///
/// The fields in this struct are only relevant when making a request to the `boomlings` servers.
/// When using GDCF with a custom Geometry Dash API, they can safely be ignored.
#[derive(Debug, Clone, Hash)]
pub struct BaseRequest {
    /// The version of the game client we're pretending to be
    ///
    /// The value of this field doesn't matter, and the request will succeed regardless of
    /// what it's been set to
    pub game_version: GameVersion,

    /// Internal version of the game client we're pretending to be
    ///
    /// The value of this field doesn't matter, and the request will succeed regardless of
    /// what it's been set to
    pub binary_version: GameVersion,

    /// The current secret String the server uses to identify valid clients.
    ///
    /// Settings this field to an incorrect value will cause the request to fail
    pub secret: String,
}

impl BaseRequest {
    /// Constructs a new `BaseRequest` with the given values.
    pub fn new(
        game_version: GameVersion,
        binary_version: GameVersion,
        secret: String,
    ) -> BaseRequest {
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

pub trait Request: Display + Default {
    type Result;

    fn new() -> Self {
        Default::default()
    }
}

pub trait MakeRequest: Request {
    fn make<C: ApiClient>(&self, client: &C) -> ApiFuture<C::Err>;
}