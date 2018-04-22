use diesel::deserialize::Queryable;
use diesel::pg::Pg;
use gdcf::model::song::NewgroundsSong;
use diesel::insert_into;
use diesel::ExpressionMethods;
use diesel::connection::Connection;
use diesel::RunQueryDsl;
use diesel::QueryDsl;
use diesel::delete;

use chrono::NaiveDateTime;

use schema::_O;
use gdcf::cache::CachedObject;

table! {
    newgrounds_song (song_id) {
        song_id -> BigInt,
        song_name -> Text,
        artist -> Text,
        filesize -> Double,
        alt_artist -> Nullable<Text>,
        banned -> Bool,
        download_link -> Text,
        internal_id -> BigInt,
        first_cached_at -> Timestamp,
        last_cached_at -> Timestamp,
    }
}

impl Queryable<newgrounds_song::SqlType, Pg> for _O<NewgroundsSong>
{
    type Row = (i64, String, String, f64, Option<String>, bool, String, i64, NaiveDateTime, NaiveDateTime);

    fn build(row: Self::Row) -> Self {
        let song = NewgroundsSong {
            song_id: row.0 as u64,
            name: row.1,
            artist: row.2,
            filesize: row.3,
            alt_artist: row.4,
            banned: row.5,
            link: row.6,
            internal_id: row.7 as u64,
        };

        CachedObject::new(song, row.8, row.9).into()
    }
}