use crate::wrap::Wrapped;
use diesel::{
    associations::Identifiable,
    backend::Backend,
    deserialize::{FromSqlRow, Queryable},
    sql_types::*,
    ExpressionMethods,
};
use gdcf_model::song::NewgroundsSong;

impl<'a> Identifiable for &'a Wrapped<NewgroundsSong> {
    type Id = &'a u64;

    fn id(self) -> Self::Id {
        &self.0.song_id
    }
}

diesel_stuff! {
    newgrounds_song (song_id, NewgroundsSong) {
        (song_id, song_id, u64),
        (song_name, name, String),
        (index_3, index_3,  u64),
        (song_artist, artist, String),
        (filesize, filesize, f64),
        (index_6, index_6, Option<String>),
        (index_7, index_7, Option<String>),
        (index_8, index_8, String),
        (song_link, link, String)
    }
}

meta_table!(song_meta, song_id);

store_simply!(NewgroundsSong, newgrounds_song, song_meta, song_id);
lookup_simply!(NewgroundsSong, newgrounds_song, song_meta, song_id);
