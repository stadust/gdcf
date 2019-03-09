//! Module containing utility functions or struct-impls that are valid across
//! (most) backends

use crate::core::{
    backend::Database,
    statement::{Preparation, Prepare},
    QueryPart,
};

pub fn join_statements<'a, DB: 'a, QP: 'a, I>(stmts: I, seperator: Option<&str>) -> Preparation<'a, DB>
where
    DB: Database,
    QP: QueryPart<DB>,
    I: IntoIterator<Item = &'a QP>,
{
    let mut p = Preparation::<DB>::default();
    let mut sep = None;

    for t in stmts {
        if let Some(seperator) = sep {
            p = p.with_static(seperator);
        }

        p = p.with(t.to_sql());

        sep = seperator;
    }

    p
}
