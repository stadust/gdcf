#![feature(trace_macros)]
#![feature(macro_at_most_once_rep)]

//trace_macros!(true);

extern crate gdcf;
#[cfg(feature = "pg")]
extern crate postgres;

use core::backend::Database;
//use core::backend::pg::Pg;
use core::query::create::Column;
use core::query::create::Create;
use core::query::Insert;
use core::query::QueryPart;
use core::query::select::Select;
use gdcf::model::NewgroundsSong;

#[macro_use]
mod core;
pub mod schema;

pub fn test() {
    use schema::song::newgrounds_song::*;

    /*let ins = Insert::new(
        &table,
        vec![
            song_id.set(&5),
            song_name.set(&"Hello")
        ]);

    println!("{}", ins.to_sql_unprepared());
    println!("{}", ins.to_sql().0.to_statement(|idx| format!("${}", idx)));

    let select: Select<Pg> = table
        .filter(song_id.eq(&5))
        .filter(song_name.same_as(&song_artist))
        .select(vec![&song_name])
        .limit(5);

    let c: Create<Pg> = create();

    //println!("{:?}", select);*/
}
