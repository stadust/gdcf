#[macro_export]
macro_rules! backend_abstraction {
    ($schema: ident) => {
        #[cfg(feature = "postgres")]
        mod pg;
        #[cfg(feature = "mysql")]
        mod mysql;
        #[cfg(feature = "sqlite")]
        mod sqlite;

        #[cfg(feature = "postgres")]
        pub use self::pg::*;
        #[cfg(feature = "mysql")]
        pub use self::mysql::*;
        #[cfg(feature = "sqlite")]
        pub use self::sqlite::*;

        #[cfg(feature = "postgres")]
        use self::pg::$schema::dsl::*;
        #[cfg(feature = "mysql")]
        use self::mysql::$schema::dsl::*;
        #[cfg(feature = "sqlite")]
        use self::sqlite::$schema::dsl::*;

        #[cfg(feature = "postgres")]
        use diesel::pg::Pg as _Backend;
        #[cfg(feature = "mysql")]
        use diesel::mysql::Mysql as _Backend;
        #[cfg(feature = "sqlite")]
        use diesel::sqlite::Sqlite as _Backend;
    }
}