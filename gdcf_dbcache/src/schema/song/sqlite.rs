use diesel::deserialize::Queryable;
use schema::song::Song;
use gdcf::model::song::NewgroundsSong;
use diesel::sqlite::Sqlite;
use diesel::prelude::SqliteConnection;
use diesel::insert_into;

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
    }
}

impl Queryable<newgrounds_song::SqlType, Sqlite> for Song
{
    type Row = (i64, String, String, f64, Option<String>, i16, String, i64);

    fn build(row: Self::Row) -> Self {
        Song(NewgroundsSong {
            song_id: row.0 as u64,
            name: row.1,
            artist: row.2,
            filesize: row.3,
            alt_artist: row.4,
            banned: row.5 != 0,
            link: row.6,
            internal_id: row.7 as u64,
        })
    }
}