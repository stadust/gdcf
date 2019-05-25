#[cfg(feature = "pg")]
macro_rules! upsert {
    ($self: expr, $object: expr, $table: expr, $column: expr) => {{
        diesel::insert_into($table)
            .values($object)
            .on_conflict($column)
            .do_update()
            .set(Wrapped($object))
            .execute(&$self.pool.get()?)?;
    }};
}

macro_rules! update_entry {
    ($self: expr, $entry: expr, $table: expr, $column: expr) => {{
        use diesel::{ExpressionMethods, QueryDsl};

        diesel::delete($table.filter($column.eq($entry.key as i64))).execute(&$self.pool.get()?)?;
        diesel::insert_into($table).values($entry).execute(&$self.pool.get()?)?;
    }};
}

#[cfg(feature = "sqlite")]
macro_rules! upsert {
    ($self: expr, $object: expr, $table: expr, $_: expr) => {
        diesel::replace_into($table).values($object).execute(&$self.pool.get()?)?;
    };
}

macro_rules! __diesel_type {
    (i64) => {Int8};
    (u64) => {Int8};
    (i32) => {Int4};
    (u32) => {Int4};
    (i16) => {Int2};
    (u16) => {Int2};
    (i8) => {Int2};
    (u8) => {Int2};
    (f64) => {Double};
    (bool) => {Bool};
    (String) => {Text};
    (Option<$t: ident>) => {Nullable<__diesel_type!($t)>};
    (Vec<u8>) => {Binary};
    (LevelRating) => {Text};
    (LevelLength) => {Text};
    (Password) => {Nullable<Text>};
    (Featured) => {Int4};
    (GameVersion) => {Int2};
    (MainSong) => {Int2};
}

macro_rules! __ref_if_not_copy {
    (i64) => {i64};
    (i32) => {i32};
    (i16) => {i16};
    (i8) => {i16};
    (u64) => {i64};
    (u32) => {i32};
    (u16) => {i16};
    (u8) => {i16};
    (f64) => {f64};
    (f32)  => {f32};
    (bool) => {bool};
    (String) => {&'a str};
    (Option<String>) => {Option<&'a str>};
    (Option<$t: ident>) => {Option<__ref_if_not_copy!($t)>};
    (Vec<u8>) => {&'a [u8]};
    (LevelRating) => {String};
    (LevelLength) => {String};
    (Password) => {Option<&'a str>};
    (Featured) => {i32};
    (GameVersion) => {i16};
    (MainSong) => {i16};
}

macro_rules! __row_type {
    (i64) => {i64};
    (i32) => {i32};
    (i16) => {i16};
    (i8) => {i16};
    (u64) => {i64};
    (u32) => {i32};
    (u16) => {i16};
    (u8) => {i16};
    (f64) => {f64};
    (f32)  => {f32};
    (bool) => {bool};
    (String) => {String};
    (Option<$t: ident>) => {Option<__row_type!($t)>};
    (Vec<u8>) => {Vec<u8>};
    (LevelRating) => {String};
    (LevelLength) => {String};
    (Password) => {Option<String>};
    (Featured) => {i32};
    (GameVersion) => {i16};
    (MainSong) => {i16};
}

macro_rules! __for_queryable {
    ($value: expr, u8) => {
        $value as u8
    };
    ($value: expr, u16) => {
        $value as u16
    };
    ($value: expr, u32) => {
        $value as u32
    };
    ($value: expr, u64) => {
        $value as u64
    };
    ($value: expr, Option<MainSong>) => {
        $value.map(|i| From::from(i as u8))
    };
    ($value: expr, Option<$t: ident>) => {
        $value.map(|inner| __for_queryable!(inner, $t))
    };
    ($value: expr, LevelRating) => {
        LevelRating::from($value)
    };
    ($value: expr, LevelLength) => {
        LevelLength::from($value)
    };
    ($value: expr, Password) => {
        match $value {
            None => Password::NoCopy,
            Some(pw) =>
                if pw == "1" {
                    Password::FreeCopy
                } else {
                    Password::PasswordCopy(pw)
                },
        }
    };
    ($value: expr, Featured) => {{
        Featured::from($value)
    }};
    ($value: expr, GameVersion) => {{
        GameVersion::from($value as u8)
    }};
    ($value: expr, $($t:tt)*) => {
        $value
    };
}

macro_rules! __for_values {
    ($value: expr, u8) => {
        $value as i16
    };
    ($value: expr, u16) => {
        $value as i16
    };
    ($value: expr, u32) => {
        $value as i32
    };
    ($value: expr, u64) => {
        $value as i64
    };
    ($value: expr, i8) => {
        $value as i16
    };
    ($value: expr, i16) => {
        $value
    };
    ($value: expr, i32) => {
        $value
    };
    ($value: expr, i64) => {
        $value
    };
    ($value: expr, f32) => {
        $value
    };
    ($value: expr, f64) => {
        $value
    };
    ($value: expr, bool) => {
        $value
    };
    ($value: expr, String) => {
        &$value[..]
    };
    ($value: expr, Option<String>) => {{
        // Compiler couldn't figure out some mysterious type 'T'
        // So now they're all annotated
        // This whole thing is pretty weird tbh
        let v: &Option<String> = &$value;
        let v: Option<&String> = v.as_ref();
        v.map(|inner: &String| &inner[..])
    }};
    ($value: expr, Option<MainSong>) => {
        $value.map(|song| song.main_song_id as i16)
    };
    ($value: expr, Option<$t: ident>) => {
        $value.map(|inner| __for_values!(inner, $t))
    };
    ($value: expr, Vec<u8>) => {
        &$value[..]
    };
    ($value: expr, LevelRating) => {
        $value.to_string()
    };
    ($value: expr, LevelLength) => {
        $value.to_string()
    };
    ($value: expr, Password) => {
        match $value {
            Password::NoCopy => None,
            Password::FreeCopy => Some("1"),
            Password::PasswordCopy(ref password) => Some(password.as_ref()),
        }
    };
    ($value: expr, Featured) => {{
        let value: i32 = $value.into();
        value // y'all gay
    }};
    ($value: expr, GameVersion) => {{
        let byte: u8 = $value.into();
        byte as i16
    }};
    ($value: expr, $($t:tt)*) => {
        &$value
    };
}

macro_rules! store_simply {
    ($to_store_ty: ty, $table: ident, $meta: ident, $primary: ident) => {
        fn __impl_store() {
            use crate::{meta::Entry, Cache};
            use diesel::RunQueryDsl;
            use gdcf::cache::Store;
            use log::debug;

            impl Store<$to_store_ty> for Cache {
                fn store(&mut self, object: &$to_store_ty, key: u64) -> Result<Entry, Self::Err> {
                    debug!("Storing {}", object);

                    let entry = Entry::new(key);

                    update_entry!(self, entry, $meta::table, $meta::$primary);
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
            use crate::{wrap::Wrapped, Cache};
            use diesel::{QueryDsl, RunQueryDsl};
            use gdcf::cache::{CacheEntry, Lookup};

            impl Lookup<$to_lookup_ty> for Cache {
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
    ($table_name: ident($primary_key: ident, $rust_ty: ty) {$(($column_name: ident, $field_name: ident, $($rust_type:tt)*)),*}) => {
        table! {
            $table_name($primary_key) {
                $($column_name -> __diesel_type!($($rust_type)*),)*
            }
        }

        impl diesel::associations::HasTable for crate::wrap::Wrapped<$rust_ty> {
            type Table = $table_name::table;

            fn table() -> Self::Table {
                $table_name::table
            }
        }

        type Row = ($(__row_type!($($rust_type)*)),*);
        type Values<'a> = ($(diesel::dsl::Eq<$table_name::$column_name, __ref_if_not_copy!($($rust_type)*)>),*);
        type SqlType = ($(__diesel_type!($($rust_type)*)),*);

        impl<'a> diesel::Insertable<$table_name::table> for &'a $rust_ty {
            type Values = <Values<'a> as diesel::Insertable<$table_name::table>>::Values;

            fn values(self) -> Self::Values {
                use self::values;

                values(self).values()
            }
        }

        #[allow(unused_imports)]
        use diesel::sql_types::*;

        /*
        Alright, so

        Wrapped( $rust_ty {
            ...
        })

        doesn't work because macro variables are weird like that, so we need this very noisy workaround
        */
        trait __ConstructExt {
            fn __construct(row: Row) -> Self;
        }

        impl __ConstructExt for $rust_ty {
            fn __construct(($($field_name),*): Row) -> Self {
                Self {
                    $(
                        $field_name: __for_queryable!($field_name, $($rust_type)*)
                    ),*
                }
            }
        }

        impl<DB: Backend> Queryable<SqlType, DB> for Wrapped<$rust_ty>
        where
            Row: FromSqlRow<SqlType, DB>,
        {
            type Row = Row;

            fn build(row: Self::Row) -> Self {
                Wrapped(<$rust_ty>::__construct(row))
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

        fn values(object: &$rust_ty) -> Values {
            use $table_name::columns::*;

            (
                $(
                    $column_name.eq(__for_values!(object.$field_name, $($rust_type)*))
                ),*
            )
        }
    };
}
