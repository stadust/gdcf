//! Module ontianing request definitions for retrieving users

use crate::api::request::{BaseRequest, Request, GD_21};
use gdcf_model::user::User;
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

impl Display for UserRequest {
    fn fmt(&self, f: &mut Formatter) -> Result<(), Error> {
        write!(f, "UserRequest({})", self.user)
    }
}

impl Request for UserRequest {
    type Result = User;
}
