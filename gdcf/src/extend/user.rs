use crate::{api::request::UserRequest, cache::Cache, extend::Extendable};
use gdcf_model::user::{SearchedUser, User};

impl<C: Cache> Extendable<C, User, User> for SearchedUser {
    type Request = UserRequest;

    fn lookup_extension(&self, cache: &C, request_result: User) -> Result<User, <C as Cache>::Err> {
        Ok(request_result)
    }

    fn on_extension_absent() -> Option<User> {
        None
    }

    fn extension_request(&self) -> Self::Request {
        self.account_id.into()
    }

    fn extend(self, user: User) -> User {
        user
    }

    fn change_extension(current: User, new_extension: User) -> User {
        new_extension
    }
}
