use core::Database;
use core::table::SetField;
use core::table::Table;

#[derive(Debug)]
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