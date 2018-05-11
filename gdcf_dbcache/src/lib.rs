#![feature(trace_macros)]

//trace_macros!(true);

extern crate gdcf;
extern crate postgres;

use core::query::Insert;
use core::query::QueryPart;
use gdcf::model::NewgroundsSong;
use core::query::create::Create;
use core::query::create::Column;

#[macro_use]
mod core;

table! {
    newgrounds_song => {
        song_id, song_name, index_3, song_artist, filesize, index_6, index_7, index_8, song_link, first_cached_at, last_cached_at
    }
}

insertable! {
    NewgroundsSong => newgrounds_song {
        /*song_id => song_id,
        name => song_name
        index_3 => index_3,
        artist => song_artist,
        index_6 => index_6,
        index_7 => index_7,
        index_8 => index_8,
        link => song_link*/
    }
}


pub fn test() {
    use newgrounds_song::*;

    let ins = Insert::new(
        &table,
        vec![
            song_id.set(&5),
            song_name.set(&"Hello")
        ]);

    println!("{}", ins.to_sql_unprepared());
    println!("{}", ins.to_sql().0.to_statement(|idx| format!("${}", idx)));

    let select = table
        .filter(song_id.eq(&5))
        .filter(song_name.same_as(&song_artist))
        .select(vec![&song_name])
        .limit(5);

    let create = Create::new("newgrounds_song")
        .ignore_if_exists()
        .with_column(Column::new("song_id", "BIGINT").primary())
        .with_column(Column::new("song_name", "TEXT"));

    println!("{:?}", select);
}