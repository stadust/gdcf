use gdcf::model::song::NewgroundsSong;

use diesel::connection::Connection;
use diesel::replace_into;
use diesel::QueryDsl;
use diesel::RunQueryDsl;
use diesel::ExpressionMethods;
use diesel::insert_into;

use schema::DBCached;
use gdcf::cache::CachedObject;
use chrono::{NaiveDateTime, Utc};

backend_abstraction!(newgrounds_song);

pub struct Song(pub NewgroundsSong, pub NaiveDateTime, pub NaiveDateTime);

into!(Song, NewgroundsSong);


impl DBCached<_Backend, u64> for Song {
    type Inner = NewgroundsSong;

    get!(|sid| newgrounds_song.find(sid as i64), u64, "postgres", "sqlite");
    get!(|sid| newgrounds_song.find(sid), u64, "mysql");

    insert!(|song: NewgroundsSong| {
        (
            song_id.eq(song.song_id as i64),
            song_name.eq(song.name),
            artist.eq(song.artist),
            filesize.eq(song.filesize),
            song.alt_artist.map(|aa| alt_artist.eq(aa)),
            banned.eq(song.banned as i16),
            download_link.eq(song.link),
            internal_id.eq(song.internal_id as i64),
            last_cached_at.eq(Utc::now().naive_utc())
        )
    }, "sqlite");

    insert!(|song: NewgroundsSong| {
        (
            song_id.eq(song.song_id),
            song_name.eq(song.name),
            artist.eq(song.artist),
            filesize.eq(song.filesize),
            song.alt_artist.map(|aa| alt_artist.eq(aa)),
            banned.eq(song.banned),
            download_link.eq(song.link),
            internal_id.eq(song.internal_id),
            last_cached_at.eq(Utc::now().naive_utc())
        )
    }, "mysql");

    pg_insert!(|song: NewgroundsSong| {
        (
            song_id.eq(song.song_id as i64),
            song_name.eq(song.name),
            artist.eq(song.artist),
            filesize.eq(song.filesize),
            song.alt_artist.map(|aa| alt_artist.eq(aa)),
            banned.eq(song.banned),
            download_link.eq(song.link),
            internal_id.eq(song.internal_id as i64),
            last_cached_at.eq(Utc::now().naive_utc())
        )
    });
}