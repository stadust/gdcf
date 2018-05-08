use chrono::{NaiveDateTime, Utc};
use diesel::connection::Connection;
use diesel::insert_into;
use diesel::replace_into;
use diesel::ExpressionMethods;
use diesel::QueryDsl;
use diesel::RunQueryDsl;
use diesel::result::Error;
use gdcf::cache::CachedObject;
use gdcf::model::song::NewgroundsSong;
use schema::_O;
use schema::Cached;

schema! { NewgroundsSong[newgrounds_song (song_id)],
    song_id[2]: u64 => (i64[BigInt], i64[BigInt], u64[Unsigned<BigInt>]),
    song_name[3, name]: String[Text],
    index_3[4]: u64 => (i64[BigInt], i64[BigInt], u64[Unsigned<BigInt>]),
    artist[5]: String[Text],
    filesize[6]: f64[Double],
    index_6[7]: Option<String>[Nullable<Text>],
    index_7[8]: Option<String>[Nullable<Text>],
    index_8[9]: i32[Integer],
    download_link[10, link]: String[Text],
}

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
            index_3.eq(song.index_3 as i64),
            artist.eq(song.artist),
            filesize.eq(song.filesize),
            song.index_6.map(|i|index_6.eq(i)),
            song.index_7.map(|i|index_7.eq(i)),
            index_8.eq(song.index_8),
            download_link.eq(song.link),
            last_cached_at.eq(Utc::now().naive_utc())
        ),
        "sqlite"
    );

    store!(
        |song: NewgroundsSong| (
            song_id.eq(song.song_id),
            song_name.eq(song.name),
            index_3.eq(song.index_3),
            artist.eq(song.artist),
            filesize.eq(song.filesize),
            song.index_6.map(|i|index_6.eq(i)),
            song.index_7.map(|i|index_7.eq(i)),
            index_8.eq(song.index_8),
            download_link.eq(song.link),
            last_cached_at.eq(Utc::now().naive_utc())
        ),
        "mysql"
    );

    pg_store!(
        |song: NewgroundsSong| (
            song_id.eq(song.song_id as i64),
            song_name.eq(song.name),
            index_3.eq(song.index_3 as i64),
            artist.eq(song.artist),
            filesize.eq(song.filesize),
            song.index_6.map(|i|index_6.eq(i)),
            song.index_7.map(|i|index_7.eq(i)),
            index_8.eq(song.index_8),
            download_link.eq(song.link),
            last_cached_at.eq(Utc::now().naive_utc())
        )
    );
}