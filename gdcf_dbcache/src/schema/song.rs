use gdcf::model::NewgroundsSong;
// use schema::NowAtUtc;

table! {
    NewgroundsSong => newgrounds_song {
        song_id => song_id,
        name => song_name,
        index_3 => index_3,
        artist => song_artist,
        filesize => filesize,
        index_6 => index_6,
        index_7 => index_7,
        index_8 => index_8,
        link => song_link;
        first_cached_at,
        last_cached_at
    }
}

create! { newgrounds_song,
    song_id[NotNull, Unique, Primary] => Unsigned<BigInteger>,
    song_name[NotNull] => Text,
    index_3 => Unsigned<BigInteger>,
    song_artist[NotNull] => Text,
    filesize[NotNull] => Double,
    index_6 => Text,
    index_7 => Text,
    index_8 => Integer,
    song_link[NotNull] => Text,
    first_cached_at[NotNull] => UtcTimestamp,
    last_cached_at[NotNull] => UtcTimestamp
}
