#![deny(
bare_trait_objects, missing_debug_implementations, unused_extern_crates, patterns_in_fns_without_body, stable_features, unknown_lints, unused_features, unused_imports, unused_parens
)]

extern crate itertools;
extern crate proc_macro;

use create::Create;
use proc_macro::TokenStream;
use table::Table;

#[macro_use]
mod macros;
mod table;
mod create;

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
        tab.gated_impl("sqlite", "sqlite", "Sqlite"),
        tab.gated_impl("mysql", "mysql", "Mysql")
    }
}

#[proc_macro]
pub fn itable(ts: TokenStream) -> TokenStream {
    let tab = Table::parse(ts);

    stream! {
        tab.generate(),
        tab.gated_insertable_impl("pg", "pg", "Pg"),
        tab.gated_insertable_impl("sqlite", "sqlite", "Sqlite"),
        tab.gated_insertable_impl("mysql", "mysql", "Mysql")
    }
}

#[proc_macro]
pub fn create(ts: TokenStream) -> TokenStream {
    Create::parse(ts).generate()
}