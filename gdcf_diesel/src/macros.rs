#[cfg(feature = "pg")]
macro_rules! upsert {
    ($self: expr, $object: expr, $table: expr, $column: expr) => {{
        use diesel::{pg::upsert::*, query_builder::AsChangeset};

        diesel::insert_into($table)
            .values($object)
            .on_conflict($column)
            .do_update()
            .set(Wrapped($object))
            .execute(&$self.pool.get()?)?;
    }};
}

#[cfg(feature = "sqlite")]
macro_rules! upsert {
    ($self: expr, $object: expr, $table: expr, $_: expr) => {
        diesel::replace_into($table).values($object).execute(&$self.pool.get()?)?;
    };
}

macro_rules! store_simply {
    ($to_store_ty: ty, $table: ident, $meta: ident, $primary: ident) => {
        fn __impl_store() {
            use crate::{meta::Entry, Cache, DB};
            use diesel::{Insertable, RunQueryDsl};
            use gdcf::cache::Store;

            impl Store<$to_store_ty> for Cache<DB> {
                fn store(&mut self, object: &$to_store_ty, key: u64) -> Result<Entry, Self::Err> {
                    let entry = Entry::new(key);

                    entry.insert_into($meta::table).execute(&self.pool.get()?)?;

                    upsert!(self, object, $table::table, $table::$primary);

                    Ok(entry)
                }
            }
        }
    };
}

macro_rules! lookup_simply {
    ($to_lookup_ty: ty, $object_table: ident,  $meta_table: ident, $primary_column: ident) => {
        fn __impl_lookup() {
            use crate::{meta::Entry, wrap::Wrapped, Cache, DB};
            use diesel::{QueryDsl, RunQueryDsl};
            use gdcf::cache::{CacheEntry, Lookup};

            impl Lookup<$to_lookup_ty> for Cache<DB> {
                fn lookup(&self, key: u64) -> Result<CacheEntry<$to_lookup_ty, Self>, Self::Err> {
                    let connection = self.pool.get()?;
                    let entry = $meta_table::table
                        .filter($meta_table::$primary_column.eq(key as i64))
                        .get_result(&connection)?;
                    let entry = self.entry(entry);
                    let wrapped: Wrapped<$to_lookup_ty> = $object_table::table
                        .filter($object_table::$primary_column.eq(key as i64))
                        .get_result(&connection)?;

                    Ok(CacheEntry::new(wrapped.0, entry))
                }
            }
        }
    };
}

macro_rules! diesel_stuff {
    ($table_name: ident($primary_key: ident, $rust_ty: ty) {$(($column_name: ident, $diesel_type: ty, $rust_type: ty, $borrowed_rust_type: ty)),*}) => {
        table! {
            $table_name($primary_key) {
                $($column_name -> $diesel_type,)*
            }
        }

        impl diesel::associations::HasTable for crate::wrap::Wrapped<$rust_ty> {
            type Table = $table_name::table;

            fn table() -> Self::Table {
                $table_name::table
            }
        }

        use diesel::sql_types::*;

        type Row = ($($rust_type),*);
        type Values<'a> = ($(diesel::dsl::Eq<$table_name::$column_name, $borrowed_rust_type>),*);
        type SqlType = ($($diesel_type),*);

        impl<'a> diesel::Insertable<$table_name::table> for &'a $rust_ty {
            type Values = <Values<'a> as diesel::Insertable<$table_name::table>>::Values;

            fn values(self) -> Self::Values {
                use self::values;

                values(self).values()
            }
        }

        impl<'a> diesel::query_builder::AsChangeset for crate::wrap::Wrapped<&'a $rust_ty> {
            type Changeset = <Values<'a> as diesel::query_builder::AsChangeset>::Changeset;
            type Target = $table_name::table;

            fn as_changeset(self) -> Self::Changeset {
                use self::values;

                values(&self.0).as_changeset()
            }
        }
    };
}
