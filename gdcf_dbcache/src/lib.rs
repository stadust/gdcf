#![feature(trace_macros)]
#![feature(macro_at_most_once_rep)]

#![deny(
bare_trait_objects, missing_debug_implementations, unused_extern_crates, patterns_in_fns_without_body, stable_features, unknown_lints, unused_features, unused_imports, unused_parens
)]


//trace_macros!(true);

extern crate chrono;
extern crate gdcf;
#[cfg(feature = "pg")]
extern crate postgres;

#[macro_use]
mod core;
mod ser;
mod de;
pub mod schema;
pub mod cache;

pub fn test() {/*
    let c: Create<Pg> = song::create();

    println!("{}", c.to_sql_unprepared());
    println!("{}", c.to_sql().0.to_statement(|idx| format!("${}", idx)));

    let c2: Create<Pg> = level::create()
        .ignore_if_exists();

    println!("{}", c2.to_sql_unprepared());
*/
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
