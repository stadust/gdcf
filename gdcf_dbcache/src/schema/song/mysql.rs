use diesel::deserialize::Queryable;
use schema::song::Song;
use gdcf::model::song::NewgroundsSong;
use diesel::mysql::Mysql;
use diesel::mysql::MysqlConnection;
use diesel::insert_into;

table! {
    newgrounds_song (song_id) {
        song_id -> Unsigned<BigInt>,
        name -> Text,
        artist -> Text,
        filesize -> Double,
        alt_artist -> Nullable<Text>,
        banned -> Bool,
        link -> Text,
    }
}

impl Queryable<newgrounds_song::SqlType, Mysql> for Song
{
    type Row = (u64, String, String, f64, Option<String>, bool, String);

    fn build(row: Self::Row) -> Self {
        Song(NewgroundsSong {
            song_id: row.0,
            name: row.1,
            artist: row.2,
            filesize: row.3,
            alt_artist: row.4,
            banned: row.5,
            link: row.6,
        })
    }
}

pub fn insert(song: NewgroundsSong, conn: &MysqlConnection)
{
    use super::newgrounds_song::dsl::*;

    insert_into(newgrounds_song)
        .values((
            song_id.eq(song.song_id),
            name.eq(song.name),
            artist.eq(song.artist),
            filesize.eq(song.filesize),
            song.alt_artist.map(|aa| alt_artist.eq(aa)),
            banned.eq(song.banned),
            link.eq(song.link)
        ))
        .execute(conn);
}