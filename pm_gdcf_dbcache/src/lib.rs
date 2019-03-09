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

extern crate itertools;
extern crate proc_macro;

use crate::{create::Create, table::Table};
use proc_macro::TokenStream;

#[macro_use]
mod macros;
mod create;
mod table;

#[proc_macro]
pub fn table(ts: TokenStream) -> TokenStream {
    Table::parse(ts).generate()
}

#[proc_macro]
pub fn iqtable(ts: TokenStream) -> TokenStream {
    let tab = Table::parse(ts);

    stream! {
        tab.generate(),
        tab.gated_impl("pg", "pg", "Pg"),
        tab.gated_impl("sqlite", "sqlite", "Sqlite")
    }
}

#[proc_macro]
pub fn itable(ts: TokenStream) -> TokenStream {
    let tab = Table::parse(ts);

    stream! {
        tab.generate(),
        tab.gated_insertable_impl("pg", "pg", "Pg"),
        tab.gated_insertable_impl("sqlite", "sqlite", "Sqlite")
    }
}

#[proc_macro]
pub fn create(ts: TokenStream) -> TokenStream {
    Create::parse(ts).generate()
}
