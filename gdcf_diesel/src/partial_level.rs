use crate::wrap::Wrapped;
use diesel::{backend::Backend, deserialize::FromSqlRow, insertable::Insertable, ExpressionMethods, Queryable};
use gdcf_model::{
    level::{Featured, LevelLength, LevelRating, PartialLevel},
    GameVersion,
};

diesel_stuff! {
    partial_level (level_id, PartialLevel<Option<u64>, u64>) {
        (level_id, level_id, u64),
        (level_name, name, String),
        (description, description, Option<String>),
        (level_version, version, u32),
        (creator_id, creator, u64),
        (difficulty, difficulty, LevelRating),
        (downloads, downloads, u32),
        (main_song, main_song, Option<MainSong>),
        (gd_version, gd_version, GameVersion),
        (likes, likes, i32),
        (level_length, length, LevelLength),
        (stars, stars, u8),
        (featured, featured, Featured),
        (index_31, index_31, Option<String>),
        (copy_of, copy_of, Option<u64>),
        (custom_song_id, custom_song, Option<u64>),
        (coin_amount, coin_amount, u8),
        (coins_verified, coins_verified, bool),
        (stars_requested, stars_requested, Option<u8>),
        (index_40, index_40, Option<String>),
        (is_epic, is_epic, bool),
        (index_43, index_43, String),
        (object_amount, object_amount, Option<u32>),
        (index_46, index_46, Option<String>),
        (index_47, index_47, Option<String>)
    }
}

// Metadata table storing information about when a partial level was cached
meta_table!(partial_level_meta, level_id);

store_simply!(PartialLevel<Option<u64>, u64>, partial_level, partial_level_meta, level_id);
lookup_simply!(PartialLevel<Option<u64>, u64>, partial_level, partial_level_meta, level_id);

// Metadata table associating the hashes of cached requests with the level ids the requested
// returned
table! {
    level_request_results (level_id, request_hash) {
        level_id -> Int8,
        request_hash -> Int8,
    }
}

// # WTF
impl Insertable<level_request_results::table> for (u64, u64) {
    type Values = <(
        diesel::dsl::Eq<level_request_results::level_id, i64>,
        diesel::dsl::Eq<level_request_results::request_hash, i64>,
    ) as Insertable<level_request_results::table>>::Values;

    fn values(self) -> Self::Values {
        (
            level_request_results::level_id.eq(self.0 as i64),
            level_request_results::request_hash.eq(self.1 as i64),
        )
            .values()
    }
}

// Metadata table storing information about when a whole request result set was cached
meta_table!(level_list_meta, request_hash);

allow_tables_to_appear_in_same_query!(level_request_results, partial_level);

joinable!(level_request_results -> partial_level(level_id));
