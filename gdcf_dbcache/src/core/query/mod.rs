use core::{AsSql, Database, statement::PreparedStatement, table::{Field, Table}};
use core::table::SetField;
use self::condition::Condition;
use core::query::condition::And;

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

pub(crate) struct Insert<'a, DB: Database + 'a> {
    table: &'a Table,
    values: Vec<SetField<'a, DB>>,
}

impl<'a, DB: Database + 'a> Insert<'a, DB> {
    pub(crate) fn new(table: &'a Table, values: Vec<SetField<'a, DB>>) -> Insert<'a, DB> {
        Insert {
            table,
            values,
        }
    }

    pub(crate) fn values(&self) -> &Vec<SetField<'a, DB>> {
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
    filter: Option<&'a Condition<'a, DB>>,
}

/*impl<'a, DB: Database + 'a> Select<'a, DB> {
    fn filter<C: Condition<'a, DB>>(self, cond: &'a C) -> Self
        where
            And<'a, DB>: QueryPart<'a, DB>
    {
        self.filter = match self.filter {
            None => Some(cond),
            Some(cond2) => Some(&cond.and(cond2))
        };

        self
    }
}*/

pub(crate) trait Insertable<DB: Database> {
    fn values<'a>(&'a self) -> Vec<SetField<'a, DB>>;

    fn insert_into<'a>(&'a self, table: &'a Table) -> Result<Insert<'a, DB>, DB::Error> {
        Ok(
            Insert {
                table,
                values: self.values(),
            }
        )
    }
}

pub(crate) trait Selectable<DB: Database> {}