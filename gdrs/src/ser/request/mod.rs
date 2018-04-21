use gdcf::model::GameVersion;
use gdcf::api::request::BaseRequest;

use ser;

pub(super) mod level;

#[derive(Serialize)]
#[serde(remote = "BaseRequest")]
pub struct BaseRequestRem {
    #[serde(rename = "gameVersion", serialize_with = "ser::game_version")]
    pub game_version: GameVersion,

    #[serde(rename = "binaryVersion", serialize_with = "ser::game_version")]
    pub binary_version: GameVersion,
    pub secret: String,
}