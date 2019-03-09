use crate::core::{
    backend::Database,
    query::{
        condition::{And, Condition},
        Query,
    },
    table::Table,
    QueryPart,
};

#[derive(Debug)]
pub struct Delete<'a, DB: Database + 'a> {
    pub table: &'a Table,
    pub filter: Option<Box<dyn Condition<DB>>>,
}

impl<'a, DB: Database + 'a> Delete<'a, DB> {
    pub fn new(table: &'a Table) -> Delete<'a, DB> {
        Delete { table, filter: None }
    }

    pub fn if_met<Cond>(mut self, cond: Cond) -> Delete<'a, DB>
    where
        DB: 'static,
        Cond: Condition<DB> + 'static,
        And<DB>: Condition<DB>,
    {
        self.filter = match self.filter {
            None => Some(Box::new(cond)),
            Some(old) =>
                Some(Box::new(And {
                    cond_1: old,
                    cond_2: Box::new(cond),
                })),
        };

        self
    }
}

if_query_part!(Delete<'a, DB>, Query<DB>);
