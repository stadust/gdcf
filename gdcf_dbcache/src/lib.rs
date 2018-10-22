#![deny(
    bare_trait_objects,
    missing_debug_implementations,
    unused_extern_crates,
    patterns_in_fns_without_body,
    stable_features,
    unknown_lints,
    unused_features,
    unused_imports,
    unused_parens
)]

extern crate gdcf;
#[macro_use]
extern crate log;
extern crate joinery;
extern crate pm_gdcf_dbcache;
#[cfg(feature = "pg")]
extern crate postgres;
#[cfg(feature = "sqlite")]
extern crate rusqlite;
extern crate seahash;
extern crate failure;
#[macro_use]
extern crate failure_derive;

#[macro_use]
mod core;
pub mod cache;
mod de;
pub mod resulter;
pub mod schema;
mod ser;
mod util;
