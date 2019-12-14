use crate::{api::request::UserRequest, cache::Cache, upgrade::Upgradable};
use gdcf_model::user::{SearchedUser, User};
use crate::cache::Lookup;

impl Upgradable<User> for SearchedUser {
    type From = SearchedUser;
    type Request = UserRequest;
    type Upgrade = User;
    type Lookup = !;

    fn upgrade_request(&self) -> Option<Self::Request> {
        Some(self.account_id.into())
    }

    fn default_upgrade() -> Option<Self::Upgrade> {
        None
    }

    fn lookup_upgrade<C: Cache + Lookup<Self::Lookup>>(&self, _: &C, request_result: User) -> Result<Self::Upgrade, <C as Cache>::Err> {
        Ok(request_result)
    }

    fn upgrade(self, upgrade: Self::Upgrade) -> (User, SearchedUser) {
        (upgrade, self)
    }

    fn downgrade(upgraded: User, downgrade: Self::From) -> (Self, Self::Upgrade) {
        (downgrade, upgraded)
    }
}
