use crate::wrap::Wrapped;
use core::borrow::Borrow;
use diesel::{
    associations::{HasTable, Identifiable},
    backend::Backend,
    deserialize::{FromSqlRow, Queryable},
    query_builder::AsChangeset,
    sql_types::*,
    ExpressionMethods, Insertable,
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
/*
impl<DB: Backend> Queryable<SqlType, DB> for Wrapped<NewgroundsSong>
where
    Row: FromSqlRow<SqlType, DB>,
{
    type Row = Row;

    fn build(row: Self::Row) -> Self {
        Wrapped(NewgroundsSong {
            song_id: row.0 as u64,
            name: row.1,
            index_3: row.2 as u64,
            artist: row.3,
            filesize: row.4,
            index_6: row.5,
            index_7: row.6,
            index_8: row.7,
            link: row.8,
        })
    }
}
*/