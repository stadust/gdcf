use gdcf::{
    api::request::{
        level::{LevelRequestType, LevelsRequest, SearchFilters},
        BaseRequest, LevelRequest,
    },
    model::{DemonRating, LevelLength, LevelRating},
};

use super::BaseRequestRem;
use ser;

#[derive(Serialize)]
#[serde(remote = "LevelRequest")]
pub struct LevelRequestRem {
    #[serde(flatten, with = "BaseRequestRem")]
    base: BaseRequest,

    #[serde(rename = "levelID")]
    level_id: u64,

    #[serde(serialize_with = "ser::bool_to_int")]
    inc: bool,

    #[serde(serialize_with = "ser::bool_to_int", rename = "extras")]
    extra: bool,
}

#[derive(Debug, Default, Serialize)]
#[serde(remote = "LevelsRequest")]
pub struct LevelsRequestRem {
    #[serde(flatten, with = "BaseRequestRem")]
    base: BaseRequest,

    #[serde(rename = "type", serialize_with = "ser::req_type")]
    request_type: LevelRequestType,

    #[serde(rename = "str")]
    search_string: String,

    #[serde(rename = "len", serialize_with = "ser::length_vec")]
    pub lengths: Vec<LevelLength>,

    #[serde(rename = "diff", serialize_with = "ser::rating_vec")]
    ratings: Vec<LevelRating>,

    #[serde(
        rename = "demonFilter",
        skip_serializing_if = "Option::is_none",
        serialize_with = "ser::demon_rating"
    )]
    pub demon_rating: Option<DemonRating>,

    pub page: u32,

    pub total: i32,

    #[serde(flatten, serialize_with = "ser::search_filters")]
    pub search_filters: SearchFilters,
}
