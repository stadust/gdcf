#![feature(trace_macros)]

//trace_macros!(true);

#[macro_use]
extern crate postgres;
extern crate gdcf;

#[macro_use]
mod core;

use core::query::Insert;
use core::query::QueryPart;

table! {
    newgrounds_song => {
        song_id, song_name
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
}