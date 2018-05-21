use core::backend::Database;
use core::backend::Error;
use core::FromSql;
use core::query::condition::And;
use core::query::condition::Condition;
use core::query::Query;
use core::query::QueryPart;
use core::table::Field;
use core::table::Table;

#[derive(Debug)]
pub struct Join<'a, DB: Database + 'a> {
    other: &'a Table,
    join_condition: &'a Condition<'a, DB>,
}

#[derive(Ord, PartialOrd, PartialEq, Eq, Copy, Clone, Debug)]
pub enum Ordering {
    Asc,
    Desc,
}

#[derive(Debug)]
pub struct OrderBy<'a> {
    field: &'a Field,
    ordering: Ordering,
}

#[derive(Debug)]
pub struct Select<'a, DB: Database + 'a> {
    table: &'a Table,
    fields: Vec<&'a Field>,
    joins: Vec<Join<'a, DB>>,
    filter: Option<Box<Condition<'a, DB>>>,
    subset: (Option<usize>, Option<usize>),
    order: Vec<OrderBy<'a>>,
}

impl<'a, DB: Database + 'a> Select<'a, DB> {
    pub fn new(table: &'a Table, fields: Vec<&'a Field>) -> Select<'a, DB> {
        Select {
            table,
            fields,
            joins: Vec::new(),
            filter: None,
            subset: (None, None),
            order: Vec::new(),
        }
    }

    pub fn limit(mut self, limit: usize) -> Select<'a, DB> {
        self.subset = (self.subset.0, Some(limit));
        self
    }

    pub fn offset(mut self, offset: usize) -> Select<'a, DB> {
        self.subset = (Some(offset), self.subset.1);
        self
    }

    pub fn select(mut self, fields: Vec<&'a Field>) -> Select<'a, DB> {
        self.fields = fields;
        self
    }

    pub fn order_by(mut self, field: &'a Field, ordering: Ordering) -> Select<'a, DB> {
        self.order.push(OrderBy { field, ordering });
        self
    }

    pub fn filter<C: 'a>(mut self, cond: C) -> Select<'a, DB>
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

pub struct Row<DB: Database> {
    fields: Vec<DB::Types>,
}

impl<DB: Database> Row<DB> {
    pub fn new(values: Vec<DB::Types>) -> Row<DB> {
        Row {
            fields: values
        }
    }

    pub fn len(&self) -> usize {
        self.fields.len()
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    pub fn get<T>(&self, idx: isize) -> Option<Result<T, Error<DB>>>
        where
            T: FromSql<DB>
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
    fn select_from(table: &Table) -> Select<DB> {
        table.select()
    }

    fn from_row(row: &Row<DB>, offset: isize) -> Result<Self, Error<DB>>;
}

impl<'a, DB: Database + 'a> Query<'a, DB> for Select<'a, DB>
    where
        Select<'a, DB>: QueryPart<'a, DB> {}