use chrono::{NaiveDateTime, Utc};
use diesel::connection::Connection;
use diesel::insert_into;
use diesel::replace_into;
use diesel::ExpressionMethods;
use diesel::QueryDsl;
use diesel::RunQueryDsl;
use gdcf::cache::CachedObject;
use gdcf::model::song::NewgroundsSong;
use schema::_O;
use schema::Cached;

backend_abstraction!(newgrounds_song);

impl Cached<_Backend> for CachedObject<NewgroundsSong> {
    type SearchKey = u64;
    type Inner = NewgroundsSong;

    retrieve!(|sid| newgrounds_song.find(sid as i64), "postgres", "sqlite");
    retrieve!(|sid| newgrounds_song.find(sid), "mysql");

    store!(
        |song: NewgroundsSong| (
            song_id.eq(song.song_id as i64),
            song_name.eq(song.name),
            artist.eq(song.artist),
            filesize.eq(song.filesize),
            song.alt_artist.map(|aa| alt_artist.eq(aa)),
            banned.eq(song.banned as i16),
            download_link.eq(song.link),
            internal_id.eq(song.internal_id as i64),
            last_cached_at.eq(Utc::now().naive_utc())
        ),
        "sqlite"
    );

    store!(
        |song: NewgroundsSong| (
            song_id.eq(song.song_id),
            song_name.eq(song.name),
            artist.eq(song.artist),
            filesize.eq(song.filesize),
            song.alt_artist.map(|aa| alt_artist.eq(aa)),
            banned.eq(song.banned),
            download_link.eq(song.link),
            internal_id.eq(song.internal_id),
            last_cached_at.eq(Utc::now().naive_utc())
        ),
        "mysql"
    );

    pg_store!(|song: NewgroundsSong| (
        song_id.eq(song.song_id as i64),
        song_name.eq(song.name),
        artist.eq(song.artist),
        filesize.eq(song.filesize),
        song.alt_artist.map(|aa| alt_artist.eq(aa)),
        banned.eq(song.banned),
        download_link.eq(song.link),
        internal_id.eq(song.internal_id as i64),
        last_cached_at.eq(Utc::now().naive_utc())
    ));
}