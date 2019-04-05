use crate::wrap::Wrapped;
use diesel::{backend::Backend, deserialize::FromSqlRow, insertable::Insertable, ExpressionMethods, Queryable};
use gdcf_model::{
    level::{Featured, LevelLength, LevelRating, PartialLevel},
    song::MainSong,
    GameVersion,
};

diesel_stuff! {
    partial_level (level_id, PartialLevel<u64, u64>) {
        (level_id, Int8, i64, i64),
        (level_name, Text, String, &'a String),
        (description, Nullable<Text>, Option<String>, &'a Option<String>),
        (level_version, Int4, i32, i32),
        (creator_id, Int8, i64, i64),
        (difficulty, Text, String, String),
        (downloads, Int4, i32, i32),
        (main_song, Nullable<Int2>, Option<i16>, Option<i16>),
        (gd_version, Int2, i16, i16),
        (likes, Int4, i32, i32),
        (level_length, Text, String, String),
        (stars, Int2, i16, i16),
        (featured, Int4, i32, i32),
        (copy_of, Nullable<Int8>, Option<i64>, Option<i64>),
        (custom_song_id, Nullable<Int8>, Option<i64>, Option<i64>),
        (coin_amount, Int2, i16, i16),
        (coins_verified, Bool, bool, bool),
        (stars_requested, Nullable<Int2>, Option<i16>, Option<i16>),
        (is_epic, Bool, bool, bool),
        (index_43, Text, String, &'a String),
        (object_amount, Nullable<Int4>, Option<i32>, Option<i32>),
        (index_46, Text, String, &'a String),
        (index_47, Text, String, &'a String)
    }
}

fn values(level: &PartialLevel<u64, u64>) -> Values {
    use partial_level::columns::*;

    (
        level_id.eq(level.level_id as i64),
        level_name.eq(&level.name),
        description.eq(&level.description),
        level_version.eq(level.version as i32),
        creator_id.eq(level.creator as i64),
        difficulty.eq(level.difficulty.to_string()),
        downloads.eq(level.downloads as i32),
        main_song.eq(level.main_song.map(|song| song.main_song_id as i16)),
        gd_version.eq(Into::<u8>::into(level.gd_version) as i16),
        likes.eq(level.likes),
        level_length.eq(level.length.to_string()),
        stars.eq(level.stars as i16),
        featured.eq(Into::<i32>::into(level.featured)),
        copy_of.eq(level.copy_of.map(|i| i as i64)),
        custom_song_id.eq(level.custom_song.map(|i| i as i64)),
        coin_amount.eq(level.coin_amount as i16),
        coins_verified.eq(level.coins_verified),
        stars_requested.eq(level.stars_requested.map(|u| u as i16)),
        is_epic.eq(level.is_epic),
        index_43.eq(&level.index_43),
        object_amount.eq(level.object_amount.map(|i| i as i32)),
        index_46.eq(&level.index_46),
        index_47.eq(&level.index_47),
    )
}

meta_table!(partial_level_meta, level_id);

store_simply!(PartialLevel<u64, u64>, partial_level, partial_level_meta, level_id);
lookup_simply!(PartialLevel<u64, u64>, partial_level, partial_level_meta, level_id);

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
}

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
