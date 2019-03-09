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

#[macro_use]
mod core;
pub mod cache;
mod de;
pub mod resulter;
pub mod schema;
mod ser;
