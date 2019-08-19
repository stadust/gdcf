use crate::{api::request::UserRequest, cache::Cache, upgrade::Upgrade};
use gdcf_model::user::{SearchedUser, User};

impl<C: Cache> Upgrade<C, User> for SearchedUser {
    type From = SearchedUser;
    type Request = UserRequest;
    type Upgrade = User;

    fn upgrade_request(from: &Self::From) -> Option<Self::Request> {
        Some(from.account_id.into())
    }

    fn current(&self) -> &Self::From {
        self
    }

    fn default_upgrade() -> Option<Self::Upgrade> {
        None
    }

    fn lookup_upgrade(from: &Self::From, cache: &C, request_result: User) -> Result<Self::Upgrade, <C as Cache>::Err> {
        Ok(request_result)
    }

    fn upgrade(self, upgrade: Self::Upgrade) -> User {
        upgrade
    }
}
