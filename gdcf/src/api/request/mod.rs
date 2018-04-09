use model::level::GameVersion;
use serde::Serializer;

pub mod level;
mod ser;

#[derive(Serialize, Debug)]
pub struct BaseRequest {
    #[cfg_attr(feature="gj-format", serde(rename = "gameVersion"))]
    #[serde(serialize_with = "ser::game_version")]
    game_version: GameVersion,

    #[cfg_attr(feature="gj-format", serde(rename = "binaryVersion"))]
    #[serde(serialize_with = "ser::game_version")]
    binary_version: GameVersion,

    secret: String,
}

impl BaseRequest {
    pub fn new(game_version: GameVersion, binary_version: GameVersion, secret: String) -> BaseRequest {
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