use crate::wrap::Wrapped;
use diesel::{
    associations::Identifiable,
    backend::Backend,
    deserialize::FromSqlRow,
    sql_types::{Int8, Nullable, Text},
    ExpressionMethods, Queryable,
};
use gdcf_model::user::User;

impl<'a> Identifiable for &'a Wrapped<User> {
    type Id = &'a u64;

    fn id(self) -> Self::Id {
        &self.0.user_id
    }
}

diesel_stuff! {
    profile (user_id, User) {
        (username, name, String),
        (user_id, user_id, u64),
        (stars, stars, u32),
        (demons, demons, u16),
        (creator_points, creator_points, u16),
        (index_10, index_10, String),
        (index_11, index_11, String),
        (secret_coints, secret_coins, u16),
        (account_id, account_id, u64),
        (user_coins, user_coins, u16),
        (index_18, index_18, String),
        (index_19, index_19, String),
        (youtube_url, youtube_url, Option<String>),
        (cube_index, cube_index, u16),
        (ship_index, ship_index, u16),
        (ball_index, ball_index, u16),
        (ufo_index, ufo_index, u16),
        (wave_index, wave_index, u16),
        (robot_index, robot_index, u16),
        (has_glow, has_glow, bool),
        (index_29, index_29, String),
        (global_rank, global_rank, Option<u32>),
        (index_31, index_31, String),
        (spider_index, spider_index, u16),
        (twitter_url, twitter_url, Option<String>),
        (twitch_url, twitch_url, Option<String>),
        (diamonds, diamonds, u16),
        (death_effect_index, death_effect_index, u16),
        (index_49, index_49, String),
        (index_50, index_50, String)
    }
}
meta_table!(profile_meta, user_id);

store_simply!(User, profile, profile_meta, user_id);
lookup_simply!(User, profile, profile_meta, user_id);
/*
fn values(profile: &User) -> Values {
    use profile::columns::*;

    (
        username.eq(&profile.name[..]),
    )
}
*/
impl<DB: Backend> Queryable<SqlType, DB> for Wrapped<User>
where
    Row: FromSqlRow<SqlType, DB>,
{
    type Row = Row;

    fn build(row: Self::Row) -> Self {
        unimplemented!()
    }
}
