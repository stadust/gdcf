use super::BaseRequestRem;
use gdcf::api::request::{
    user::{UserRequest, UserSearchRequest},
    BaseRequest,
};
use serde_derive::Serialize;

#[derive(Serialize)]
#[serde(remote = "UserRequest")]
pub struct UserRequestRem {
    #[serde(flatten, with = "BaseRequestRem")]
    base: BaseRequest,

    #[serde(rename = "targetAccountID")]
    user: u64,
}

#[derive(Serialize)]
#[serde(remote = "UserSearchRequest")]
pub struct UserSearchRequestRem {
    #[serde(flatten, with = "BaseRequestRem")]
    base: BaseRequest,

    total: u32,

    page: u32,

    #[serde(rename = "str")]
    search_string: String,
}
