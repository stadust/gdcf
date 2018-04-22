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
        artist -> Text,
        filesize -> Double,
        alt_artist -> Nullable<Text>,
        banned -> SmallInt,
        download_link -> Text,
        internal_id -> BigInt,
        first_cached_at -> Timestamp,
        last_cached_at -> Timestamp,
    }
}

impl Queryable<newgrounds_song::SqlType, Sqlite> for _O<NewgroundsSong>
{
    type Row = (i64, String, String, f64, Option<String>, i16, String, i64, NaiveDateTime, NaiveDateTime);

    fn build(row: Self::Row) -> Self {
        let song = NewgroundsSong {
            song_id: row.0 as u64,
            name: row.1,
            artist: row.2,
            filesize: row.3,
            alt_artist: row.4,
            banned: row.5 != 0,
            link: row.6,
            internal_id: row.7 as u64,
        };

        CachedObject::new(song, row.8, row.9).into()
    }
}