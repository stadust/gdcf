use diesel::deserialize::Queryable;
use diesel::mysql::Mysql;

use schema::_O;

use gdcf::model::song::NewgroundsSong;
use gdcf::cache::CachedObject;

use diesel::mysql::MysqlConnection;
use diesel::replace_into;
use diesel::Connection;
use diesel::delete;
use diesel::QueryDsl;
use diesel::RunQueryDsl;
use diesel::ExpressionMethods;

use chrono::NaiveDateTime;

table! {
    newgrounds_song (song_id) {
        song_id -> Unsigned<BigInt>,
        song_name -> Text,
        artist -> Text,
        filesize -> Double,
        alt_artist -> Nullable<Text>,
        banned -> Bool,
        download_link -> Text,
        internal_id -> Unsigned<BigInt>,
        first_cached_at -> Timestamp,
        last_cached_at -> Timestamp,
    }
}

impl Queryable<newgrounds_song::SqlType, Mysql> for _O<NewgroundsSong>
{
    type Row = (u64, String, String, f64, Option<String>, bool, String, u64, NaiveDateTime, NaiveDateTime);

    fn build(row: Self::Row) -> Self {
        let song = NewgroundsSong {
            song_id: row.0,
            name: row.1,
            artist: row.2,
            filesize: row.3,
            alt_artist: row.4,
            banned: row.5,
            link: row.6,
            internal_id: row.7,
        };

        CachedObject::new(song, row.8, row.9).into()
    }
}