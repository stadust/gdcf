use crate::wrap::Wrapped;
use diesel::{
    associations::Identifiable,
    backend::Backend,
    deserialize::FromSqlRow,
    sql_types::{Int8, Nullable, Text},
    ExpressionMethods, Queryable,
};
use gdcf_model::user::Creator;

impl<'a> Identifiable for &'a Wrapped<Creator> {
    type Id = &'a u64;

    fn id(self) -> Self::Id {
        &self.0.user_id
    }
}

diesel_stuff! {
    creator (user_id, Creator) {
        (user_id, user_id, u64),
        (name, name, String),
        (account_id, account_id, Option<u64>)
    }
}
meta_table!(creator_meta, user_id);

store_simply!(Creator, creator, creator_meta, user_id);
lookup_simply!(Creator, creator, creator_meta, user_id);
