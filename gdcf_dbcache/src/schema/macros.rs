#[macro_export]
macro_rules! backend_abstraction {
    ($schema:ident) => {
        #[cfg(feature = "mysql")]
        mod mysql;
        #[cfg(feature = "postgres")]
        mod pg;
        #[cfg(feature = "sqlite")]
        mod sqlite;

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