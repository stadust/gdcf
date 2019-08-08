use crate::api::request::{BaseRequest, PaginatableRequest, Request, GD_21};
use gdcf_model::comment::{CommentUser, LevelComment, ProfileComment};
use std::{
    fmt::{Display, Formatter},
    hash::{Hash, Hasher},
};
//use gdcf_model::level::{PartialLevel, Level};

/// The different orderings that can be requested for level comments
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum SortMode {
    /// Sort the comments by likes, in descending order
    ///
    /// ## GD Internals:
    /// This variant is represented by the numeric value `1` in the boomlings API
    Liked,

    /// Sort the comments from newest to oldest
    ///
    /// ## GD Internals:
    /// This variant is represented by the numeric value `0` in the boomlings API
    Recent,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct LevelCommentsRequest {
    /// Whether this [`LevelCommentsRequest`] request forces a cache refresh. This is not a HTTP
    /// request field!
    pub force_refresh: bool,

    /// The base request data
    pub base: BaseRequest,

    /// Unknown, probably related to pagination
    ///
    /// ## GD Internals:
    /// This field is called `total` in the boomlings API
    pub total: u32,

    /// The page of users to retrieve. The first page is page `0`
    ///
    /// ## GD Internals:
    /// This field is called `page` in the boomlings API
    pub page: u32,

    /// What to sort by comments by
    ///
    /// ## GD Internals:
    /// This field is called `mode` in the boomlings API.
    pub sort_mode: SortMode,

    /// The id of the level to retrieve the comments of
    ///
    /// ## GD Internals:
    /// This field is called `levelID` in the boomlings API
    pub level_id: u64,

    /// The amount of comments to retrieve. Note that while in-game this can only be set to 20 or 40
    /// (via the "load more comments option), the API accepts any value. So you can set it to
    /// something ridiculously high (like u32::MAX_VALUE) and retrieve all comments at once.
    ///
    /// ## GD Internals:
    /// This field is called `count` in the boomlings API
    pub limit: u32,
}

impl LevelCommentsRequest {
    const_setter!(with_base, base, BaseRequest);

    const_setter!(total: u32);

    const_setter!(limit: u32);

    const_setter!(page: u32);

    pub const fn force_refresh(mut self) -> Self {
        self.force_refresh = true;
        self
    }

    pub const fn new(level: u64) -> LevelCommentsRequest {
        LevelCommentsRequest {
            force_refresh: false,
            level_id: level,
            base: GD_21,
            page: 0,
            total: 0,
            sort_mode: SortMode::Recent,
            limit: 20,
        }
    }

    pub const fn liked(mut self) -> Self {
        self.sort_mode = SortMode::Liked;
        self
    }

    pub const fn recent(mut self) -> Self {
        self.sort_mode = SortMode::Recent;
        self
    }
}

impl Display for LevelCommentsRequest {
    fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
        write!(f, "LevelCommentsRequest({})", self.level_id)
    }
}

impl Hash for LevelCommentsRequest {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.level_id.hash(state);
        self.sort_mode.hash(state);
        self.limit.hash(state);
        self.page.hash(state);
        self.total.hash(state);
    }
}

impl Request for LevelCommentsRequest {
    type Result = Vec<LevelComment<Option<CommentUser>>>;

    fn forces_refresh(&self) -> bool {
        self.force_refresh
    }
}

impl PaginatableRequest for LevelCommentsRequest {
    fn next(&self) -> Self {
        LevelCommentsRequest {
            page: self.page + 1,
            ..*self
        }
    }
}
// FIXME: figure out these impls
/*
impl<S: PartialEq, U: PartialEq> Into<LevelCommentsRequest> for PartialLevel<S, U> {
    fn into(self) -> LevelCommentsRequest {
        LevelCommentsRequest::new(self.level_id)
    }
}

impl<S: PartialEq, U: PartialEq> Into<LevelCommentsRequest> for Level<S, U> {
    fn into(self) -> LevelCommentsRequest {
        LevelCommentsRequest::new(self.level_id)
    }
}

impl<S: PartialEq, U: PartialEq> Into<LevelCommentsRequest> for &PartialLevel<S, U> {
    fn into(self) -> LevelCommentsRequest {
        LevelCommentsRequest::new(self.level_id)
    }
}

impl<S: PartialEq, U: PartialEq> Into<LevelCommentsRequest> for &Level<S, U> {
    fn into(self) -> LevelCommentsRequest {
        LevelCommentsRequest::new(self.level_id)
    }
}*/

impl Into<LevelCommentsRequest> for u64 {
    fn into(self) -> LevelCommentsRequest {
        LevelCommentsRequest::new(self)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ProfileCommentsRequest {
    /// Whether this [`ProfileCommentsRequest`] request forces a cache refresh. This is not a HTTP
    /// request field!
    pub force_refresh: bool,

    /// The base request data
    pub base: BaseRequest,

    /// Unknown, probably related to pagination
    ///
    /// ## GD Internals:
    /// This field is called `total` in the boomlings API
    pub total: u32,

    /// The page of users to retrieve. The first page is page `0`
    ///
    /// ## GD Internals:
    /// This field is called `page` in the boomlings API
    pub page: u32,

    /// The account id of the user to retrieve the comments of
    ///
    /// ## GD Internals:
    /// This field is called `accountID` in the boomlings API
    pub account_id: u64,
}

impl ProfileCommentsRequest {
    const_setter!(with_base, base, BaseRequest);

    const_setter!(total: u32);

    const_setter!(page: u32);

    const_setter!(account_id: u64);

    pub const fn force_refresh(mut self) -> Self {
        self.force_refresh = true;
        self
    }

    pub const fn new(account: u64) -> ProfileCommentsRequest {
        ProfileCommentsRequest {
            force_refresh: false,
            account_id: account,
            base: GD_21,
            page: 0,
            total: 0,
        }
    }
}

impl Display for ProfileCommentsRequest {
    fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
        write!(f, "AccountCommentsRequest({})", self.account_id)
    }
}

impl Hash for ProfileCommentsRequest {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.account_id.hash(state);
        self.page.hash(state);
        self.total.hash(state);
    }
}

impl Request for ProfileCommentsRequest {
    type Result = Vec<ProfileComment>;

    fn forces_refresh(&self) -> bool {
        self.force_refresh
    }
}

impl PaginatableRequest for ProfileCommentsRequest {
    fn next(&self) -> Self {
        ProfileCommentsRequest {
            page: self.page + 1,
            ..*self
        }
    }
}
