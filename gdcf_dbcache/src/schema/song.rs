pub(crate) mod newgrounds_song {
    use crate::core::backend::Error;
    use gdcf_model::song::NewgroundsSong;

    use crate::schema::NowAtUtc;

    use pm_gdcf_dbcache::{create, iqtable};

    iqtable! {
        NewgroundsSong => newgrounds_song {
            song_id => song_id,
            name => song_name,
            index_3 => index_3,
            artist => song_artist,
            filesize => filesize,
            index_6 => index_6,
            index_7 => index_7,
            index_8 => index_8,
            link => song_link,
            first_cached_at,
            last_cached_at
        }
    }

    create! {
        newgrounds_song => {
            song_id: Unsigned<BigInteger> NotNull Unique Primary,
            song_name: Text NotNull,
            index_3: Unsigned<BigInteger>,
            song_artist: Text NotNull,
            filesize: Double NotNull,
            index_6: Text,
            index_7: Text,
            index_8: Text,
            song_link: Text NotNull,
            first_cached_at: UtcTimestamp Default<NowAtUtc>(NowAtUtc) NotNull,
            last_cached_at: UtcTimestamp NotNull
        }
    }
}
