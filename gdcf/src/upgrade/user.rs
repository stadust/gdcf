use crate::{
    api::request::{Request, UserRequest},
    cache::{Cache, CacheEntry, CacheEntryMeta, Lookup},
    upgrade::{Upgradable, UpgradeError, UpgradeQuery},
};
use gdcf_model::user::{SearchedUser, User};

impl Upgradable<User> for SearchedUser {
    type From = SearchedUser;
    type LookupKey = UserRequest;
    type Request = UserRequest;
    type Upgrade = User;

    fn query_upgrade<C: Cache + Lookup<Self::LookupKey>>(
        &self,
        cache: &C,
        ignored_cached: bool,
    ) -> Result<UpgradeQuery<Self::Request, Self::Upgrade>, UpgradeError<C::Err>> {
        let mut request: UserRequest = self.account_id.into();

        request.set_force_refresh(ignored_cached);

        query_upgrade!(cache, request, request)
    }

    fn process_query_result<C: Cache + Lookup<Self::LookupKey>>(
        &self,
        cache: &C,
        resolved_query: UpgradeQuery<CacheEntry<User, C::CacheEntryMeta>, Self::Upgrade>,
    ) -> Result<UpgradeQuery<!, Self::Upgrade>, UpgradeError<C::Err>> {
        match resolved_query.one() {
            (None, Some(user)) => Ok(UpgradeQuery::One(None, Some(user))),
            (Some(CacheEntry::Cached(user, _)), _) => Ok(UpgradeQuery::One(None, Some(user))),
            _ => Err(UpgradeError::UpgradeFailed),
        }
    }

    fn upgrade<State>(self, upgrade: UpgradeQuery<State, Self::Upgrade>) -> (User, UpgradeQuery<State, Self::From>) {
        (upgrade.one().1.unwrap(), UpgradeQuery::One(None, Some(self)))
    }

    fn downgrade<State>(upgraded: User, downgrade: UpgradeQuery<State, Self::From>) -> (Self, UpgradeQuery<State, Self::Upgrade>) {
        (downgrade.one().1.unwrap(), UpgradeQuery::One(None, Some(upgraded)))
    }
}
