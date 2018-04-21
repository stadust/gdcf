use diesel::deserialize::Queryable;
use diesel::pg::Pg;
use schema::song::Song;
use gdcf::model::song::NewgroundsSong;
use diesel::insert_into;
use diesel::ExpressionMethods;
use diesel::connection::Connection;
use diesel::RunQueryDsl;
use diesel::QueryDsl;
use diesel::delete;

use schema::Cached;

table! {
    newgrounds_song (song_id) {
        song_id -> BigInt,
        name -> Text,
        artist -> Text,
        filesize -> Double,
        alt_artist -> Nullable<Text>,
        banned -> Bool,
        link -> Text,
    }
}

impl Queryable<newgrounds_song::SqlType, Pg> for Song
{
    type Row = (i64, String, String, f64, Option<String>, bool, String);

    fn build(row: Self::Row) -> Self {
        Song(NewgroundsSong {
            song_id: row.0 as u64,
            name: row.1,
            artist: row.2,
            filesize: row.3,
            alt_artist: row.4,
            banned: row.5,
            link: row.6,
        })
    }
}

impl Cached<Pg, u64> for Song {
    fn get<Conn>(sid: u64, conn: &Conn) -> Option<Self>
        where
            Conn: Connection<Backend=Pg>
    {
        use schema::song::pg::newgrounds_song::dsl::*;

        let result = newgrounds_song.find(sid as i64)
            .first(conn);

        match result {
            Ok(song) => Some(song),
            Err(_) => None
        }
    }

    fn insert<Conn>(self, conn: &Conn)
        where
            Conn: Connection<Backend=Pg>
    {
        use schema::song::pg::newgrounds_song::dsl::*;

        insert_into(newgrounds_song)
            .values((
                song_id.eq(self.0.song_id as i64),
                name.eq(self.0.name),
                artist.eq(self.0.artist),
                filesize.eq(self.0.filesize),
                self.0.alt_artist.map(|aa| alt_artist.eq(aa)),
                banned.eq(self.0.banned),
                link.eq(self.0.link)
            ))
            .execute(conn)
            .unwrap();
    }

    fn replace_with<Conn>(self, new: Song, conn: &Conn)
        where
            Conn: Connection<Backend=Pg>
    {
        use schema::song::pg::newgrounds_song::dsl::*;

        delete(newgrounds_song.filter(song_id.eq(self.0.song_id as i64)))
            .execute(conn)
            .unwrap();

        new.insert(conn);
    }
}
