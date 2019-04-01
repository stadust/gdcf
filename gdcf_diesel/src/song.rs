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
        (song_id, Int8, i64, i64),
        (song_name, Text, String, &'a String),
        (index_3, Int8, i64, i64),
        (song_artist, Text, String, &'a String),
        (filesize, Double, f64, f64),
        (index_6, Nullable<Text>, Option<String>, &'a Option<String>),
        (index_7, Nullable<Text>, Option<String>, &'a Option<String>),
        (index_8, Text, String, &'a String),
        (song_link, Text, String, &'a String)
    }
}

meta_table!(song_meta, song_id);

store_simply!(NewgroundsSong, newgrounds_song, song_meta, song_id);
lookup_simply!(NewgroundsSong, newgrounds_song, song_meta, song_id);

fn values(song: &NewgroundsSong) -> Values {
    use newgrounds_song::columns::*;

    (
        song_id.eq(song.song_id as i64),
        song_name.eq(&song.name),
        index_3.eq(song.index_3 as i64),
        song_artist.eq(&song.artist),
        filesize.eq(song.filesize),
        index_6.eq(&song.index_6),
        index_7.eq(&song.index_7),
        index_8.eq(&song.index_8),
        song_link.eq(&song.link),
    )
}

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
