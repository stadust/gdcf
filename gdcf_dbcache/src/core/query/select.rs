use crate::core::{
    backend::{Database, Error},
    query::{
        condition::{And, Condition, Or},
        Query,
    },
    table::{Field, Table},
    FromSql, QueryPart,
};

#[derive(Debug)]
pub struct Join<DB: Database> {
    pub other: Table,
    pub join_condition: Box<dyn Condition<DB>>,
}

#[derive(Ord, PartialOrd, PartialEq, Eq, Copy, Clone, Debug)]
pub enum Ordering {
    Asc,
    Desc,
}

#[derive(Debug)]
pub struct OrderBy {
    pub field: Field,
    pub ordering: Ordering,
}

#[derive(Debug)]
pub struct Select<DB: Database> {
    pub table: Table,
    pub fields: Vec<Field>,
    pub joins: Vec<Join<DB>>,
    pub filter: Option<Box<dyn Condition<DB>>>,
    pub subset: (Option<usize>, Option<usize>),
    pub order: Vec<OrderBy>,
}

impl<DB: Database> Select<DB> {
    pub fn new(table: Table, fields: Vec<Field>) -> Select<DB> {
        Select {
            table,
            fields,
            joins: Vec::new(),
            filter: None,
            subset: (None, None),
            order: Vec::new(),
        }
    }

    pub fn join<Cond>(mut self, other: Table, condition: Cond) -> Select<DB>
    where
        DB: 'static,
        Cond: Condition<DB> + 'static,
    {
        self.joins.push(Join {
            other,
            join_condition: Box::new(condition),
        });
        self
    }

    pub fn limit(mut self, limit: usize) -> Select<DB> {
        self.subset = (self.subset.0, Some(limit));
        self
    }

    pub fn offset(mut self, offset: usize) -> Select<DB> {
        self.subset = (Some(offset), self.subset.1);
        self
    }

    pub fn select(mut self, fields: &[Field]) -> Select<DB> {
        self.fields.extend(fields);
        self
    }

    pub fn order_by(mut self, field: Field, ordering: Ordering) -> Select<DB> {
        self.order.push(OrderBy { field, ordering });
        self
    }

    pub fn filter<Cond>(mut self, cond: Cond) -> Select<DB>
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

    pub fn or<Cond>(mut self, cond: Cond) -> Select<DB>
    where
        DB: 'static,
        Cond: Condition<DB> + 'static,
        Or<DB>: Condition<DB>,
    {
        self.filter = match self.filter {
            None => Some(Box::new(cond)),
            Some(old) =>
                Some(Box::new(Or {
                    cond_1: old,
                    cond_2: Box::new(cond),
                })),
        };

        self
    }
}

#[derive(Debug)]
pub struct Row<DB: Database> {
    fields: Vec<DB::Types>,
}

impl<DB: Database> Row<DB> {
    pub fn new(values: Vec<DB::Types>) -> Row<DB> {
        Row { fields: values }
    }

    pub fn len(&self) -> usize {
        self.fields.len()
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    pub fn get<T>(&self, idx: isize) -> Option<Result<T, Error<DB>>>
    where
        T: FromSql<DB>,
    {
        let idx: usize = if idx < 0 {
            (self.fields.len() as isize + idx) as usize
        } else {
            idx as usize
        };

        self.fields.get(idx).map(T::from_sql)
    }
}

pub trait Queryable<DB: Database>: Sized {
    fn select_from(table: Table) -> Select<DB> {
        table.select()
    }

    fn from_row(row: &Row<DB>, offset: isize) -> Result<Self, Error<DB>>;
}

if_query_part!(Select<DB>, Query<DB>);
