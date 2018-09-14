//! Module ontianing request definitions for retrieving users

use api::request::{BaseRequest, Request};
use std::hash::{Hash, Hasher};
use std::fmt::Display;
use std::fmt::Formatter;
use std::fmt::Error;

/// Struct modelled after a request to `getGJProfile20.php`.
///
/// In the geometry Dash API, this endpoint is used to download player profiles from the servers by
/// their account IDs
#[derive(Debug, Default, Clone)]
pub struct UserRequest {
    /// The base request data
    pub base: BaseRequest,

    /// The **account ID** of the users whose data to retrieve.
    ///
    /// ## GD Internals:
    /// This field is called `targetAccountID` in the boomlings API
    pub user: u64,
}

impl UserRequest {
    pub fn new(user_id: u64) -> UserRequest {
        UserRequest {
            base: BaseRequest::gd_21(),
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

impl Request for UserRequest {}
