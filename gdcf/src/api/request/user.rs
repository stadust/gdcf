//! Module ontianing request definitions for retrieving users

use crate::api::request::{BaseRequest, PaginatableRequest, Request, GD_21};
use gdcf_model::user::{Creator, SearchedUser, User};
use std::{
    fmt::{Display, Error, Formatter},
    hash::{Hash, Hasher},
};

/// Struct modelled after a request to `getGJUserInfo20.php`.
///
/// In the geometry Dash API, this endpoint is used to download player profiles from the servers by
/// their account IDs
#[derive(Debug, Default, Clone, Copy)]
pub struct UserRequest {
    /// The base request data
    pub base: BaseRequest,

    /// The **account ID** (_not_ user ID) of the users whose data to retrieve.
    ///
    /// ## GD Internals:
    /// This field is called `targetAccountID` in the boomlings API
    pub user: u64,
}

impl UserRequest {
    const_setter!(with_base, base, BaseRequest);

    pub const fn new(user_id: u64) -> UserRequest {
        UserRequest {
            base: GD_21,
            user: user_id,
        }
    }
}

impl Hash for UserRequest {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.user.hash(state)
    }
}

impl Into<UserRequest> for u64 {
    fn into(self) -> UserRequest {
        UserRequest::new(self)
    }
}

impl Into<UserRequest> for SearchedUser {
    fn into(self) -> UserRequest {
        UserRequest::new(self.account_id)
    }
}

impl Into<UserRequest> for &SearchedUser {
    fn into(self) -> UserRequest {
        UserRequest::new(self.account_id)
    }
}

impl Display for UserRequest {
    fn fmt(&self, f: &mut Formatter) -> Result<(), Error> {
        write!(f, "UserRequest({})", self.user)
    }
}

impl Request for UserRequest {
    type Result = User;
}

#[derive(Debug, Clone)]
pub struct UserSearchRequest {
    /// The base request data
    pub base: BaseRequest,

    /// Unknown, probably related to pagination
    ///
    /// ## GD Internals:
    /// This field is called `total` in the boomlings API
    pub total: u32,

    /// The page of users to retrieve
    ///
    /// Since the behavior of the search function was changed to return only the user whose name
    /// matches the search string exactly (previous behavior was a prefix search), it is not
    /// possible to retrieve more than 1 user via this endpoint anymore, rendering the pagination
    /// parameters useless.
    ///
    /// ## GD Internals:
    /// This field is called `page` in the boomlings API
    pub page: u32,

    /// The name of the user being searched for
    ///
    /// ## GD Internals:
    /// This field is called `str` in the boomlings API
    pub search_string: String,
}

impl UserSearchRequest {
    const_setter!(with_base, base, BaseRequest);

    const_setter!(total: u32);

    const_setter!(page: u32);

    pub const fn new(search_string: String) -> Self {
        UserSearchRequest {
            base: GD_21,
            total: 0,
            page: 0,
            search_string,
        }
    }
}

impl Into<UserSearchRequest> for String {
    fn into(self) -> UserSearchRequest {
        UserSearchRequest::new(self)
    }
}

impl Into<UserSearchRequest> for &str {
    fn into(self) -> UserSearchRequest {
        UserSearchRequest::new(self.to_string())
    }
}

impl Into<UserSearchRequest> for Creator {
    fn into(self) -> UserSearchRequest {
        UserSearchRequest::new(self.name)
    }
}

impl Into<UserSearchRequest> for &Creator {
    fn into(self) -> UserSearchRequest {
        UserSearchRequest::new(self.name.to_string())
    }
}

impl Into<UserSearchRequest> for User {
    fn into(self) -> UserSearchRequest {
        UserSearchRequest::new(self.name)
    }
}

impl Into<UserSearchRequest> for &User {
    fn into(self) -> UserSearchRequest {
        UserSearchRequest::new(self.name.to_string())
    }
}

impl Display for UserSearchRequest {
    fn fmt(&self, f: &mut Formatter) -> Result<(), Error> {
        write!(f, "UserSearchRequest({})", self.search_string)
    }
}

impl Hash for UserSearchRequest {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.search_string.hash(state)
    }
}

impl Request for UserSearchRequest {
    type Result = SearchedUser;
}

impl PaginatableRequest for UserSearchRequest {
    fn next(&self) -> Self {
        UserSearchRequest {
            base: self.base,
            total: self.total,
            page: self.page + 1,
            search_string: self.search_string.clone(),
        }
    }
}
