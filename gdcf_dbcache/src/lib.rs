#![feature(trace_macros)]

//trace_macros!(true);

#[macro_use]
extern crate diesel;
extern crate chrono;
extern crate gdcf;

#[cfg(any(feature = "postgres", feature = "mysql", feature = "sqlite"))]
pub mod cache;
pub mod schema;
