use model::GameVersion;

#[macro_use]
mod macros;
pub mod level;

pub use self::level::{LevelRequest, LevelsRequest};
use api::ApiClient;
use futures::Future;
use model::RawObject;
use api::GDError;

#[derive(Debug, Clone, Hash)]
pub struct BaseRequest {
    pub game_version: GameVersion,
    pub binary_version: GameVersion,
    pub secret: String,
}

impl BaseRequest {
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

pub trait Request {
    type Result;
}

pub(crate) trait MakeRequest: Request {
    type Item;

    fn make<C: ApiClient>(self, client: &C) -> Box<Future<Item=Self::Item, Error=GDError> + 'static>;
}