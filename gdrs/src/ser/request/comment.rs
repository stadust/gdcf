use super::BaseRequestRem;
use crate::ser;
use gdcf::api::request::{
    comment::{LevelCommentsRequest, ProfileCommentsRequest, SortMode},
    BaseRequest,
};
use serde_derive::Serialize;

#[derive(Serialize)]
#[serde(remote = "LevelCommentsRequest")]
pub struct LevelCommentsRequestRem {
    #[serde(flatten, with = "BaseRequestRem")]
    pub base: BaseRequest,

    pub total: u32,

    pub page: u32,

    #[serde(serialize_with = "ser::sort_mode", rename = "mode")]
    pub sort_mode: SortMode,

    #[serde(rename = "levelID")]
    pub level_id: u64,

    #[serde(rename = "count")]
    pub limit: u32,
}

#[derive(Serialize)]
#[serde(remote = "ProfileCommentsRequest")]
pub struct ProfileCommentsRequestRem {
    #[serde(flatten, with = "BaseRequestRem")]
    pub base: BaseRequest,

    pub total: u32,

    pub page: u32,

    #[serde(rename = "accountID")]
    pub account_id: u64,
}
