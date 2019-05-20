use crate::wrap::Wrapped;
use diesel::{backend::Backend, deserialize::FromSqlRow, insertable::Insertable, ExpressionMethods, Queryable};
use gdcf_model::{
    level::{Featured, LevelLength, LevelRating, PartialLevel},
    song::MainSong,
    GameVersion,
};
use diesel::expression::AsExpression;

diesel_stuff! {
    partial_level (level_id, PartialLevel<u64, u64>) {
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
        (copy_of, copy_of, Option<u64>),
        (custom_song_id, custom_song, Option<u64>),
        (coin_amount, coin_amount, u8),
        (coins_verified, coins_verified, bool),
        (stars_requested, stars_requested, Option<u8>),
        (is_epic, is_epic, bool),
        (index_43, index_43, String),
        (object_amount, object_amount, Option<u32>),
        (index_46, index_46, String),
        (index_47, index_47, String)
    }
}

meta_table!(partial_level_meta, level_id);

store_simply!(PartialLevel<u64, u64>, partial_level, partial_level_meta, level_id);
lookup_simply!(PartialLevel<u64, u64>, partial_level, partial_level_meta, level_id);
/*
impl<DB: Backend> Queryable<SqlType, DB> for Wrapped<PartialLevel<u64, u64>>
where
    Row: FromSqlRow<SqlType, DB>,
{
    type Row = Row;

    fn build(row: Self::Row) -> Self {
        Wrapped(PartialLevel {
            level_id: row.0 as u64,
            name: row.1,
            description: row.2,
            version: row.3 as u32,
            creator: row.4 as u64,
            difficulty: LevelRating::from(row.5),
            downloads: row.6 as u32,
            main_song: row.7.map(|i| From::from(i as u8)),
            gd_version: GameVersion::from(row.8 as u8),
            likes: row.9,
            length: LevelLength::from(row.10),
            stars: row.11 as u8,
            featured: Featured::from(row.12),
            copy_of: row.13.map(|i| i as u64),
            custom_song: row.14.map(|i| i as u64),
            coin_amount: row.15 as u8,
            coins_verified: row.16,
            stars_requested: row.17.map(|i| i as u8),
            is_epic: row.18,
            index_43: row.19,
            object_amount: row.20.map(|i| i as u32),
            index_46: row.21,
            index_47: row.22,
        })
    }
}*/

table! {
    request_results (level_id, request_hash) {
        level_id -> Int8,
        request_hash -> Int8,
    }
}

// # WTF
impl Insertable<request_results::table> for (u64, u64) {
    type Values = <(
        diesel::dsl::Eq<request_results::level_id, i64>,
        diesel::dsl::Eq<request_results::request_hash, i64>,
    ) as Insertable<request_results::table>>::Values;

    fn values(self) -> Self::Values {
        (
            request_results::level_id.eq(self.0 as i64),
            request_results::request_hash.eq(self.1 as i64),
        )
            .values()
    }
}

meta_table!(level_list_meta, request_hash);

allow_tables_to_appear_in_same_query!(request_results, partial_level);

joinable!(request_results -> partial_level(level_id));
