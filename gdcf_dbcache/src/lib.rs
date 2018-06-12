#![feature(trace_macros)]
#![feature(macro_at_most_once_rep)]

#![deny(
bare_trait_objects, missing_debug_implementations, unused_extern_crates, patterns_in_fns_without_body, stable_features, unknown_lints, unused_features, unused_imports, unused_parens
)]

// TODO: do the same thing for all the creates


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
