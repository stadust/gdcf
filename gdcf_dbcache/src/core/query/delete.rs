use core::backend::Database;
use core::query::condition::And;
use core::query::condition::Condition;
use core::query::Query;
use core::QueryPart;
use core::table::Table;

#[derive(Debug)]
pub struct Delete<'a, DB: Database + 'a> {
    pub table: &'a Table,
    pub filter: Option<Box<dyn Condition<DB>>>,
}

impl<'a, DB: Database + 'a> Delete<'a, DB> {
    pub fn new(table: &'a Table) -> Delete<'a, DB> {
        Delete {
            table,
            filter: None,
        }
    }

    pub fn if_met<Cond>(mut self, cond: Cond) -> Delete<'a, DB>
        where
            DB: 'static,
            Cond: Condition<DB> + 'static,
            And<DB>: Condition<DB>
    {
        self.filter = match self.filter {
            None => Some(Box::new(cond)),
            Some(old) => Some(Box::new(And {
                cond_1: old,
                cond_2: Box::new(cond),
            }))
        };

        self
    }
}


if_query_part!(Delete<'a, DB>, Query<DB>);
//if_sql_expr!(Delete<'a, DB>, Query<DB>);

/*
impl<'a, DB: Database + 'a> Query<DB> for Delete<'a, DB>
    where
        Delete<'a, DB>: QueryPart<DB> {}*/