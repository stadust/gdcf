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
    (ModLevel) => {Int2};
    (Color) => {Int4};
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
    (ModLevel) => {i16};
    (Color) => {i32};
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
    (ModLevel) => {i16};
    (Color) => {i32};
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
    ($value: expr, ModLevel) => {{
        ModLevel::from($value as u8)
    }};
    ($value: expr, Color) => {{
        if $value < 0 {
            Color::Unknown(-$value as u8)
        } else {
            Color::Known($value as u8, ($value >> 8) as u8, ($value >> 16) as u8)
        }
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
    ($value: expr, ModLevel) => {{
        let byte: u8 = $value.into();
        byte as i16
    }};
    ($value: expr, Color) => {{
        match $value {
            Color::Unknown(idx) => -(idx as i32),
            Color::Known(r, g, b) => r as i32 | (g as i32) << 8 | (b as i32) << 16,
        }
    }};
    ($value: expr, $($t:tt)*) => {
        &$value
    };
}

macro_rules! store_simply {
    ($key_type: ty, $table: ident, $meta: ident, $primary: ident) => {
        fn __impl_store() {
            use crate::{key::DatabaseKey, meta::Entry, Cache};
            use diesel::RunQueryDsl;
            use gdcf::cache::{Key, Store};
            use log::{debug, warn};

            impl Store<$key_type> for Cache {
                fn mark_absent(&mut self, key: &$key_type) -> Result<Entry, Self::Err> {
                    warn!("Marking {} with key {} as absent!", stringify!($key_type), key);

                    let entry = Entry::absent(key.database_key());

                    update_entry!(&self, entry, $meta::table, $meta::$primary);

                    Ok(entry)
                }

                fn store(&mut self, object: &<$key_type as Key>::Result, key: &$key_type) -> Result<Entry, Self::Err> {
                    debug!("Storing {} under key {}", object, key);

                    let entry = Entry::new(key.database_key());

                    update_entry!(self, entry, $meta::table, $meta::$primary);
                    upsert!(self, object, $table::table, $table::$primary);

                    Ok(entry)
                }
            }
        }
    };
}

macro_rules! lookup_simply {
    ($key_type: ty, $object_table: ident,  $meta_table: ident, $primary_column: ident) => {
        fn __impl_lookup() {
            use crate::{key::DatabaseKey, wrap::Wrapped, Cache, Entry};
            use diesel::{QueryDsl, RunQueryDsl};
            use gdcf::cache::{CacheEntry, Key, Lookup};
            use log::{debug, trace};

            impl Lookup<$key_type> for Cache {
                fn lookup(&self, key: &$key_type) -> Result<CacheEntry<<$key_type as Key>::Result, Entry>, Self::Err> {
                    trace!(
                        "Performing look up of {} with key {} in table {} (meta table {})",
                        stringify!($to_lookup_ty),
                        key,
                        stringify!($object_table),
                        stringify!($meta_table)
                    );

                    let connection = self.pool.get()?;
                    let entry = handle_missing!($meta_table::table
                        .filter($meta_table::$primary_column.eq(key.database_key()))
                        .get_result(&connection));
                    let entry = self.entry(entry);

                    trace!("Successfully retrieved meta entry");

                    if entry.absent {
                        debug!("Object marked as absent!");

                        return Ok(CacheEntry::MarkedAbsent(entry))
                    }

                    let wrapped: Wrapped<<$key_type as Key>::Result> = handle_missing!($object_table::table
                        .filter($object_table::$primary_column.eq(key.database_key()))
                        .get_result(&connection));

                    Ok(CacheEntry::Cached(wrapped.0, entry))
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

macro_rules! handle_missing {
    ($database_call: expr) => {
        match $database_call {
            Err(diesel::result::Error::NotFound) => {
                log::warn!("Cache miss!");

                return Ok(CacheEntry::Missing)
            },
            Err(err) => return Err($crate::Error::Database(err)),
            Ok(whatevs) => whatevs,
        }
    };
}
