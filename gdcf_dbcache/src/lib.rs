#![feature(trace_macros)]
#![feature(macro_at_most_once_rep)]
#![feature(use_extern_macros)]
#![feature(proc_macro_gen)]

#![deny(
bare_trait_objects, missing_debug_implementations, unused_extern_crates, patterns_in_fns_without_body, stable_features, unknown_lints, unused_features, unused_imports, unused_parens
)]

//trace_macros!(true);

extern crate chrono;
extern crate gdcf;
#[macro_use]
extern crate log;
extern crate pm_gdcf_dbcache;
#[cfg(feature = "pg")]
extern crate postgres;
extern crate seahash;
extern crate joinery;

#[macro_use]
mod core;
mod ser;
mod de;
mod util;
pub mod schema;
pub mod cache;
