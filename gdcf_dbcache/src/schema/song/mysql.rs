use diesel::deserialize::Queryable;
use diesel::mysql::Mysql;

use schema::song::Song;

use gdcf::model::song::NewgroundsSong;
use diesel::mysql::MysqlConnection;
use diesel::replace_into;
use schema::Cached;
use diesel::Connection;
use diesel::delete;
use diesel::QueryDsl;
use diesel::RunQueryDsl;
use diesel::ExpressionMethods;

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
    }
}

impl Queryable<newgrounds_song::SqlType, Mysql> for Song
{
    type Row = (u64, String, String, f64, Option<String>, bool, String, u64);

    fn build(row: Self::Row) -> Self {
        Song(NewgroundsSong {
            song_id: row.0,
            name: row.1,
            artist: row.2,
            filesize: row.3,
            alt_artist: row.4,
            banned: row.5,
            link: row.6,
            internal_id: row.7,
        })
    }
}