use diesel::deserialize::Queryable;
use gdcf::model::song::NewgroundsSong;
use gdcf::cache::CachedObject;
use diesel::sqlite::Sqlite;
use diesel::prelude::SqliteConnection;
use diesel::insert_into;

use chrono::NaiveDateTime;

use schema::_O;

table! {
    newgrounds_song (song_id) {
        song_id -> BigInt,
        song_name -> Text,
        index_3 -> BigInt,
        artist -> Text,
        filesize -> Double,
        index_6 -> Nullable<Text>,
        index_7 -> Nullable<Text>,
        index_8 -> Integer,
        download_link -> Text,
        first_cached_at -> Timestamp,
        last_cached_at -> Timestamp,
    }
}

impl Queryable<newgrounds_song::SqlType, Sqlite> for _O<NewgroundsSong>
{
    type Row = (i64, String, i64, String, f64, Option<String>, Option<String>, i32, String, NaiveDateTime, NaiveDateTime);

    fn build(row: Self::Row) -> Self {
        let song = NewgroundsSong {
            song_id: row.0 as u64,
            name: row.1,
            index_3: row.2 as u64,
            artist: row.3,
            filesize: row.4,
            index_6: row.5,
            index_7: row.6,
            index_8: row.7,
            link: row.8,
        };

        CachedObject::new(song, row.9, row.10).into()
    }
}