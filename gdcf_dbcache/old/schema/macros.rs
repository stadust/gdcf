#[macro_export]
macro_rules! backend_abstraction {
    ($schema:ident) => {
        #[cfg(feature = "mysql")]
        pub use self::mysql::*;
        #[cfg(feature = "postgres")]
        pub use self::pg::*;
        #[cfg(feature = "sqlite")]
        pub use self::sqlite::*;

        #[cfg(feature = "mysql")]
        use self::mysql::$schema::dsl::*;
        #[cfg(feature = "postgres")]
        use self::pg::$schema::dsl::*;
        #[cfg(feature = "sqlite")]
        use self::sqlite::$schema::dsl::*;

        #[cfg(feature = "mysql")]
        use diesel::mysql::Mysql as _Backend;
        #[cfg(feature = "postgres")]
        use diesel::pg::Pg as _Backend;
        #[cfg(feature = "sqlite")]
        use diesel::sqlite::Sqlite as _Backend;
    };
}

#[macro_export]
macro_rules! pg_store {
    ($closure:expr) => {
        #[cfg(feature = "postgres")]
        fn store<Conn>(obj: Self::Inner, conn: &Conn) -> Result<(), Error>
        where
            Conn: Connection<Backend = _Backend>,
        {
            let values = $closure(obj);

            insert_into(newgrounds_song)
                .values(&values)
                .on_conflict(song_id)
                .do_update()
                .set(values.clone())
                .execute(conn)
                .map(|_|())
        }
    };
}

#[macro_export]
macro_rules! store {
    ($closure: expr, $($db: expr),*) => {
        #[cfg(any($(feature = $db),*))]
        fn store<Conn>(obj: Self::Inner, conn: &Conn) -> Result<(), Error>
            where
                Conn: Connection<Backend=_Backend>
        {
            replace_into(newgrounds_song)
                .values($closure(obj))
                .execute(conn)
                .map(|_|())
        }
    }
}

#[macro_export]
macro_rules! retrieve {
    ($closure: expr, $($db: expr),*) => {
        #[cfg(any($(feature = $db),*))]
        fn retrieve<Conn>(key: Self::SearchKey, conn: &Conn) -> Result<Self, Error>
            where
                Conn: Connection<Backend=_Backend>
        {
            $closure(key)
                .first(conn)
                .map(|song: _O<Self::Inner>| song.into())
        }
    };
}

macro_rules! _generate {
    (
        $backend: ty, $model: ident, $db_table: ident($($prim_key: ident),*),
        $(
            $db_field: ident, $idx: tt, $field: ident, $field_type:ty, $sql_field_type: ty, $sql_type: ty
        ),*
    ) => {
        use diesel::deserialize::Queryable;

        use chrono::NaiveDateTime;

        use schema::_O;
        use gdcf::cache::CachedObject;

        table! {
            $db_table ($($prim_key)+) {
                first_cached_at -> Timestamp,
                last_cached_at -> Timestamp,
                $(
                    $db_field -> $sql_type,
                )+
            }
        }

        impl Queryable<$db_table::SqlType, $backend> for _O<$model> {
            type Row = (
                NaiveDateTime, NaiveDateTime,
                $(
                    $sql_field_type,
                )+
            );

             fn build(row: Self::Row) -> Self {
                let model = $model {
                    $(
                        $field: row.$idx as $field_type,
                    )+
                };

                CachedObject::new(model, row.0, row.1).into()
            }
        }
    };
}

macro_rules! schema {
    (@$model: ident[$db_table: ident($($prim_key: ident),*)],
        [,
            $(
                $db_field: ident[$idx: tt, $field: ident]:
                    $field_type: ty => (
                        $pg_ftype: ty[$pg_type: ty],
                        $sqlite_ftype: ty[$sqlite_type: ty],
                        $mysql_ftype: ty[$mysql_type: ty]
                    )
            ),+
        ]
    ) => {
        #[cfg(feature = "postgres")]
        mod pg {
            use super::$model;

            use diesel::pg::Pg;

            _generate!(Pg, $model, $db_table($($prim_key)*), $($db_field, $idx, $field, $field_type, $pg_ftype, $pg_type),*);
        }

        #[cfg(feature = "mysql")]
        mod mysql {
            use super::$model;

            use diesel::mysql::Mysql;

            _generate!(Mysql, $model, $db_table($($prim_key)*), $($db_field, $idx, $field, $field_type, $mysql_ftype, $mysql_type),*);
        }

        #[cfg(feature = "sqlite")]
        mod sqlite {
            use super::$model;

            use diesel::sqlite::Sqlite;

            _generate!(Sqlite, $model, $db_table($($prim_key)*), $($db_field, $idx, $field, $field_type, $sqlite_ftype, $sqlite_type),*);
        }
    };

    // field_name[row_index]: rust_type => (pg_rs_type[pg_type], sqlite_rs_type[sqlite_type], mysql_rs_type[mysql_type])
    (@$model: ident[$db_table: ident($($prim_key: ident),*)],
        [$($stack: tt)*]
        $db_field: ident[$idx: tt]:
            $field_type: ty => (
                $pg_ftype: ty[$pg_type: ty],
                $sqlite_ftype: ty[$sqlite_type: ty],
                $mysql_ftype: ty[$mysql_type: ty]
            ),
        $($rest: tt)*
    ) => {
        schema! { @$model[$db_table($($prim_key)*)],
            [
                $($stack)*,
                $db_field[$idx, $db_field]: $field_type => ($pg_ftype[$pg_type], $sqlite_ftype[$sqlite_type], $mysql_ftype[$mysql_type])
            ]
            $($rest)*
        }
    };

    // db_field_name[row_index, rust_field_name]: rust_type => (pg_rs_type[pg_type], sqlite_rs_type[sqlite_type], mysql_rs_type[mysql_type])
    (@$model: ident[$db_table: ident($($prim_key: ident),*)],
        [$($stack: tt)*]
        $db_field: ident[$idx: tt, $field: ident]:
            $field_type: ty => (
                $pg_ftype: ty[$pg_type: ty],
                $sqlite_ftype: ty[$sqlite_type: ty],
                $mysql_ftype: ty[$mysql_type: ty]
            ),
        $($rest: tt)*
    ) => {
        schema! { @$model[$db_table($($prim_key)*)],
            [
                $($stack)*,
                $db_field[$idx, $field]: $field_type => ($pg_ftype[$pg_type], $sqlite_ftype[$sqlite_type], $mysql_ftype[$mysql_type])
            ]
            $($rest)*
        }
    };

    // db_field_name[row_index, rust_field_name]: rust_type[sql_type]
    (@$model: ident[$db_table: ident($($prim_key: ident),*)],
        [$($stack: tt)*]
        $db_field: ident[$idx: tt, $field: ident]: $field_type: ty[$sql_type: ty],
        $($rest: tt)*
    ) => {
        schema! { @$model[$db_table($($prim_key)*)],
            [
                $($stack)*,
                $db_field[$idx, $field]: $field_type => ($field_type[$sql_type], $field_type[$sql_type], $field_type[$sql_type])
            ]
            $($rest)*
        }
    };

    // field_name[row_index]: rust_type[sql_type]
    (@$model: ident[$db_table: ident($($prim_key: ident),*)],
        [$($stack: tt)*]
        $db_field: ident[$idx: tt]: $field_type: ty[$sql_type: ty],
        $($rest: tt)*
    ) => {
        schema! { @$model[$db_table($($prim_key)*)],
            [
                $($stack)*,
                $db_field[$idx, $db_field]: $field_type => ($field_type[$sql_type], $field_type[$sql_type], $field_type[$sql_type])
            ]
            $($rest)*
        }
    };

    ($model: ident[$db_table: ident($($prim_key: ident),*)], $($tokens: tt)*) => {
        schema! { @$model[$db_table($($prim_key)*)],
            [] $($tokens)*
        }
    };
}