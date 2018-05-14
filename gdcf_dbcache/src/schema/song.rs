use gdcf::model::NewgroundsSong;

table! {
    NewgroundsSong => newgrounds_song {
        song_id => song_id[Unsigned<BigInteger>, NotNull],
        name => song_name[Text, NotNull],
        index_3 => index_3[Unsigned<BigInteger>],
        artist => song_artist[Text, NotNull],
        filesize => filesize[Double, NotNull],
        index_6 => index_6[Text],
        index_7 => index_7[Text],
        index_8 => index_8[Integer],
        link => song_link[Text, NotNull];
        first_cached_at[Timestamp, NotNull], last_cached_at[Text, NotNull]
    }
}
