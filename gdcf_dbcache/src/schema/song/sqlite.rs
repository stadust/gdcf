use diesel::deserialize::Queryable;
use schema::song::Song;
use gdcf::model::song::NewgroundsSong;
use diesel::sqlite::Sqlite;
use diesel::prelude::SqliteConnection;
use diesel::insert_into;

table! {
    newgrounds_song (song_id) {
        song_id -> BigInt,
        name -> Text,
        artist -> Text,
        filesize -> Double,
        alt_artist -> Nullable<Text>,
        banned -> SmallInt,
        link -> Text,
    }
}

impl Queryable<newgrounds_song::SqlType, Sqlite> for Song
{
    type Row = (i64, String, String, f64, Option<String>, i16, String);

    fn build(row: Self::Row) -> Self {
        Song(NewgroundsSong {
            song_id: row.0 as u64,
            name: row.1,
            artist: row.2,
            filesize: row.3,
            alt_artist: row.4,
            banned: row.5 != 0,
            link: row.6,
        })
    }
}

pub fn insert(song: NewgroundsSong, conn: &SqliteConnection)
{
    use super::newgrounds_song::dsl::*;

    insert_into(newgrounds_song)
        .values((
            song_id.eq(song.song_id as i64),
            name.eq(song.name),
            artist.eq(song.artist),
            filesize.eq(song.filesize),
            song.alt_artist.map(|aa| alt_artist.eq(aa)),
            banned.eq(if song.banned {1} else {0}),
            link.eq(song.link)
        ))
        .execute(conn);
}