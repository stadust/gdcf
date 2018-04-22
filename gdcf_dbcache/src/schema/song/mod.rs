use gdcf::model::song::NewgroundsSong;

use diesel::connection::Connection;
use diesel::replace_into;
use diesel::QueryDsl;
use diesel::RunQueryDsl;
use diesel::ExpressionMethods;
use diesel::insert_into;

use schema::Cached;

backend_abstraction!(newgrounds_song);

pub struct Song(pub NewgroundsSong);

pub fn update_or_insert<Conn>(song: NewgroundsSong, conn: &Conn)
    where
        Conn: Connection<Backend=_Backend>
{
    let new_song = Song(song);

    match Song::get(new_song.0.song_id, conn) {
        Some(cached) => cached.replace_with(new_song, conn),
        None => new_song.insert(conn)
    }
}


impl Cached<_Backend, u64> for Song {
    #[cfg(any(feature = "postgres", feature = "sqlite"))]
    fn get<Conn>(sid: u64, conn: &Conn) -> Option<Self>
        where
            Conn: Connection<Backend=_Backend>
    {
        let result = newgrounds_song.find(sid as i64)
            .first(conn);

        match result {
            Ok(song) => Some(song),
            Err(_) => None
        }
    }

    #[cfg(feature = "mysql")]
    fn get<Conn>(sid: u64, conn: &Conn) -> Option<Self>
        where
            Conn: Connection<Backend=_Backend>
    {
        let result = newgrounds_song.find(sid)
            .first(conn);

        match result {
            Ok(song) => Some(song),
            Err(_) => None
        }
    }

    insert!(|Song(song)|{
        (
            song_id.eq(song.song_id as i64),
            song_name.eq(song.name),
            artist.eq(song.artist),
            filesize.eq(song.filesize),
            song.alt_artist.map(|aa| alt_artist.eq(aa)),
            banned.eq(song.banned as i16),
            download_link.eq(song.link),
            internal_id.eq(song.internal_id as i64)
        )
    }, "sqlite");

    pg_insert!(|Song(song)| {
        (
            song_id.eq(song.song_id as i64),
            song_name.eq(song.name),
            artist.eq(song.artist),
            filesize.eq(song.filesize),
            song.alt_artist.map(|aa| alt_artist.eq(aa)),
            banned.eq(song.banned),
            download_link.eq(song.link),
            internal_id.eq(song.internal_id as i64)
        )
    });

    insert!(|Song(song)|{
        (
            song_id.eq(song.song_id),
            song_name.eq(song.name),
            artist.eq(song.artist),
            filesize.eq(song.filesize),
            song.alt_artist.map(|aa| alt_artist.eq(aa)),
            banned.eq(song.banned),
            download_link.eq(song.link),
            internal_id.eq(song.internal_id)
        )
    }, "mysql");
}