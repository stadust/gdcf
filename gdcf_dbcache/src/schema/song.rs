use gdcf::model::NewgroundsSong;
use core::backend::Database;
use core::query::Insertable;
use gdcf::cache::CachedObject;
use core::table::SetField;
use core::table::Table;
use core::backend::pg::Pg;

table! {
    NewgroundsSong => newgrounds_song {
        song_id => song_id[Unsigned<BigInteger>],
        name => song_name[Text],
        index_3 => index_3[Unsigned<BigInteger>],
        artist => song_artist[Text],
        index_6 => index_6[Text],
        index_7 => index_7[Text],
        index_8 => index_8[Integer],
        link => song_link[Text];
        first_cached_at[Text], last_cached_at[Text]
    }
}
