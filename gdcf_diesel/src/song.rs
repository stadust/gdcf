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

impl HasTable for Wrapped<NewgroundsSong> {
    type Table = newgrounds_song::table;

    fn table() -> Self::Table {
        newgrounds_song::table
    }
}

impl<'a> Identifiable for &'a Wrapped<NewgroundsSong> {
    type Id = &'a u64;

    fn id(self) -> Self::Id {
        &self.0.song_id
    }
}

table! {
    newgrounds_song (song_id) {
        song_id -> Int8,
        song_name -> Text,
        index_3 -> Int8,
        song_artist -> Text,
        filesize -> Double,
        index_6 -> Nullable<Text>,
        index_7 -> Nullable<Text>,
        index_8 -> Text,
        song_link -> Text,
    }
}

meta_table!(song_meta, song_id);

type NewgroundsSongRow = (i64, String, i64, String, f64, Option<String>, Option<String>, String, String);
type NewgroundsSongSqlType = (Int8, Text, Int8, Text, Double, Nullable<Text>, Nullable<Text>, Text, Text);
type NewgroundsSongValues<'a> = (
    diesel::dsl::Eq<newgrounds_song::song_id, i64>,
    diesel::dsl::Eq<newgrounds_song::song_name, &'a str>,
    diesel::dsl::Eq<newgrounds_song::index_3, i64>,
    diesel::dsl::Eq<newgrounds_song::song_artist, &'a str>,
    diesel::dsl::Eq<newgrounds_song::filesize, f64>,
    diesel::dsl::Eq<newgrounds_song::index_6, Option<&'a str>>,
    diesel::dsl::Eq<newgrounds_song::index_7, Option<&'a str>>,
    diesel::dsl::Eq<newgrounds_song::index_8, &'a str>,
    diesel::dsl::Eq<newgrounds_song::song_link, &'a str>,
);

fn values(song: &NewgroundsSong) -> NewgroundsSongValues {
    use newgrounds_song::columns::*;

    (
        song_id.eq(song.song_id as i64),
        song_name.eq(&song.name[..]),
        index_3.eq(song.index_3 as i64),
        song_artist.eq(&song.artist[..]),
        filesize.eq(song.filesize),
        index_6.eq(song.index_6.as_ref().map(AsRef::as_ref)),
        index_7.eq(song.index_6.as_ref().map(AsRef::as_ref)),
        index_8.eq(&song.index_8[..]),
        song_link.eq(&song.link[..]),
    )
}

impl<DB: Backend> Queryable<NewgroundsSongSqlType, DB> for Wrapped<NewgroundsSong>
where
    NewgroundsSongRow: FromSqlRow<NewgroundsSongSqlType, DB>,
{
    type Row = NewgroundsSongRow;

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

impl<'a> Insertable<newgrounds_song::table> for &'a NewgroundsSong {
    type Values = <NewgroundsSongValues<'a> as Insertable<newgrounds_song::table>>::Values;

    fn values(self) -> Self::Values {
        values(self).values()
    }
}

impl<'a> AsChangeset for Wrapped<&'a NewgroundsSong> {
    type Changeset = <NewgroundsSongValues<'a> as AsChangeset>::Changeset;
    type Target = newgrounds_song::table;

    fn as_changeset(self) -> Self::Changeset {
        values(&self.0).as_changeset()
    }
}
