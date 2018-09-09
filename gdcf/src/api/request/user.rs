//! Module ontianing request definitions for retrieving users

use api::request::BaseRequest;

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
