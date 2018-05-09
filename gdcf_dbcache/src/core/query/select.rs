use core::backend::Database;
use core::query::condition::And;
use core::query::condition::Condition;
use core::table::Field;
use core::table::Table;

#[derive(Debug)]
pub(crate) struct Join<'a, DB: Database + 'a> {
    other: &'a Table,
    join_condition: &'a Condition<'a, DB>,
}

#[derive(Ord, PartialOrd, PartialEq, Eq, Copy, Clone, Debug)]
pub(crate) enum Ordering {
    Asc,
    Desc,
}

#[derive(Debug)]
pub(crate) struct OrderBy<'a> {
    field: &'a Field,
    ordering: Ordering,
}

#[derive(Debug)]
pub(crate) struct Select<'a, DB: Database + 'a> {
    table: &'a Table,
    fields: Vec<&'a Field>,
    joins: Vec<Join<'a, DB>>,
    filter: Option<Box<Condition<'a, DB>>>,
    subset: (Option<usize>, Option<usize>),
    order: Vec<OrderBy<'a>>,
}

impl<'a, DB: Database + 'a> Select<'a, DB> {
    pub(crate) fn new(table: &'a Table, fields: Vec<&'a Field>) -> Select<'a, DB> {
        Select {
            table,
            fields,
            joins: Vec::new(),
            filter: None,
            subset: (None, None),
            order: Vec::new(),
        }
    }

    pub(crate) fn limit(mut self, limit: usize) -> Select<'a, DB> {
        self.subset = (self.subset.0, Some(limit));
        self
    }

    pub(crate) fn offset(mut self, offset: usize) -> Select<'a, DB> {
        self.subset = (Some(offset), self.subset.1);
        self
    }

    pub(crate) fn select(mut self, fields: Vec<&'a Field>) -> Select<'a, DB> {
        self.fields = fields;
        self
    }

    pub(crate) fn order_by(mut self, field: &'a Field, ordering: Ordering) -> Select<'a, DB> {
        self.order.push(OrderBy { field, ordering });
        self
    }

    pub(crate) fn filter<C: 'a>(mut self, cond: C) -> Select<'a, DB>
        where
            And<'a, DB>: Condition<'a, DB> + 'static,
            C: Condition<'a, DB>,
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