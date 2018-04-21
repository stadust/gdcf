use gdcf::model::song::NewgroundsSong;

use diesel::connection::Connection;
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