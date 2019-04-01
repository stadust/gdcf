use crate::wrap::Wrapped;
use diesel::{backend::Backend, deserialize::FromSqlRow, ExpressionMethods, Queryable};
use gdcf_model::level::{PartialLevel, LevelRating};
use gdcf_model::GameVersion;

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
        level_version.eq(level.level_id as i32),
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
/*
impl<DB: Backend> Queryable<SqlType, DB> for Wrapped<PartialLevel<u64, u64>>
where
    Row: FromSqlRow<SqlType, DB>,
{
    type Row = Row;

    fn build(row: Self::Row) -> Self {
        PartialLevel {
            level_id: row.0 as u64,
            name: row.1,
            description: row.2,
            version: GameVersion::from(row.3 as u8),
             creator: row.4,
            difficulty: LevelRating::from(row.5),
            downloads: row.6 as u32,
            main_song:
        }
    }
}
*/