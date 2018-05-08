use core::{AsSql, Database, statement::PreparedStatement, table::{Field, Table}};
use self::condition::Condition;

pub(crate) mod condition;

pub(crate) trait QueryPart<'a, DB: Database + 'a> {
    fn to_sql_unprepared(&self) -> String;

    fn to_sql(&self) -> (PreparedStatement, Vec<&'a AsSql<DB>>) {
        (self.to_sql_unprepared().into(), Vec::new())
    }
}

pub(crate) trait Query<'a, DB: Database + 'a>: QueryPart<'a, DB> {
    fn execute(&'a self, db: &'a DB) -> Result<(), DB::Error>
        where
            Self: Sized
    {
        db.execute(self)
    }

    fn execute_unprepared(&'a self, db: &'a DB) -> Result<(), DB::Error>
        where
            Self: Sized
    {
        db.execute_unprepared(self)
    }
}

pub(crate) enum InsertValue<'a, DB: Database + 'a> {
    Default,
    Value(&'a AsSql<DB>),
}

impl<'a, DB: Database, T: 'a> From<&'a T> for InsertValue<'a, DB>
    where
        T: AsSql<DB>
{
    fn from(t: &'a T) -> Self {
        InsertValue::Value(t)
    }
}

impl<'a, DB: Database> InsertValue<'a, DB> {
    pub(crate) fn is_default(&self) -> bool {
        match self {
            InsertValue::Default => true,
            _ => false
        }
    }

    pub(crate) fn is_value(&self) -> bool {
        match self {
            InsertValue::Value(_) => true,
            _ => false
        }
    }
}

pub(crate) struct Insert<'a, DB: Database + 'a> {
    table: &'a Table,
    values: Vec<InsertValue<'a, DB>>,
}

impl<'a, DB: Database + 'a> Insert<'a, DB> {
    pub(crate) fn values(&self) -> &Vec<InsertValue<'a, DB>> {
        &self.values
    }

    pub(crate) fn table(&self) -> &'a Table {
        self.table
    }
}

pub(crate) struct Join<'a, DB: Database + 'a> {
    other: &'a Table,
    join_condition: &'a Condition<'a, DB>,
}

pub(crate) struct Select<'a, DB: Database + 'a> {
    table: &'a Table,
    fields: Vec<&'a Field>,
    joins: Vec<Join<'a, DB>>,
    filter: &'a Condition<'a, DB>,
}

pub(crate) trait Insertable<DB: Database> {
    fn values<'a>(&'a self) -> Vec<InsertValue<'a, DB>>;

    fn insert_into<'a>(&'a self, table: &'a Table) -> Result<Insert<'a, DB>, DB::Error> {
        Ok(
            Insert {
                table,
                values: self.values(),
            }
        )
    }
}