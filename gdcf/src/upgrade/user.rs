use crate::{
    api::request::UserRequest,
    cache::{Cache, Lookup},
    upgrade::Upgradable,
};
use gdcf_model::user::{SearchedUser, User};

impl Upgradable<User> for SearchedUser {
    type From = SearchedUser;
    type LookupKey = !;
    type Request = UserRequest;
    type Upgrade = User;

    fn upgrade_request(&self) -> Option<Self::Request> {
        Some(self.account_id.into())
    }

    fn default_upgrade() -> Option<Self::Upgrade> {
        None
    }

    fn lookup_upgrade<C: Cache + Lookup<Self::LookupKey>>(&self, _: &C, request_result: User) -> Result<Self::Upgrade, <C as Cache>::Err> {
        Ok(request_result)
    }

    fn upgrade(self, upgrade: Self::Upgrade) -> (User, SearchedUser) {
        (upgrade, self)
    }

    fn downgrade(upgraded: User, downgrade: Self::From) -> (Self, Self::Upgrade) {
        (downgrade, upgraded)
    }
}
