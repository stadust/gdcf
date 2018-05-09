#![feature(trace_macros)]

//trace_macros!(true);

extern crate gdcf;
extern crate postgres;

use core::query::Insert;
use core::query::QueryPart;

#[macro_use]
mod core;

table! {
    newgrounds_song => {
        song_id, song_name, song_artist
    }
}

pub fn test() {
    use newgrounds_song::*;

    let ins = Insert::new(
        &newgrounds_song,
        vec![
            song_id.set(&5),
            song_name.set(&"Hello")
        ]);

    println!("{}", ins.to_sql_unprepared());
    println!("{}", ins.to_sql().0.to_statement(|idx| format!("${}", idx)));

    let select = newgrounds_song
        .filter(song_id.eq(&5))
        .filter(song_name.same_as(&song_artist))
        .select(vec![&song_name])
        .limit(5);

    println!("{:?}", select);
}