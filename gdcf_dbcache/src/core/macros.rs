macro_rules! table {
    ($model: ident => $table: ident {$($model_field: ident => $table_column: ident[$sql_type: ty]),*; $($unmapped_column: ident[$sql_type2: ty]),*}) => {
        pub(crate) mod $table {
            use super::$model;  // these import look weird (and they are, I dont get them), but they're needed
            use core::table::{Field, Table, SetField};
            use core::query::create::{Create, Column};
            use core::backend::Database;
            use core::query::insert::Insertable;
            use core::types::*;

            #[allow(non_upper_case_globals)]
            pub(crate) const table_name: &str = stringify!($table);

            $(
                #[allow(non_upper_case_globals)]
                pub(crate) static $table_column: Field = Field {
                    table: table_name,
                    name: stringify!($table_column)
                };
            )*

            $(
                #[allow(non_upper_case_globals)]
                pub(crate) static $unmapped_column: Field = Field {
                    table: table_name,
                    name: stringify!($unmapped_column)
                };
            )*

            #[allow(non_upper_case_globals)]
            pub(crate) static table: Table = Table {
                name: table_name,
                fields: &[
                    $(&$table_column,)*
                    $(&$unmapped_column,)*
                ]
            };

            #[cfg(feature = "pg")]
            mod pg {
                use core::backend::pg::Pg;
                use super::*;

                __insertable!(Pg, $model, $($model_field => $table_column,)*);
            }

            #[cfg(feature = "sqlite")]
            mod pg {
                use core::backend::sqlite::Sqlite;
                use super::*;

                __insertable!(Sqlite, $model, $($model_field => $table_column,)*);
            }

            #[cfg(feature = "mysql")]
            mod pg {
                use core::backend::mysql::MySql;
                use super::*;

                __insertable!(MySql, $model, $($model_field => $table_column,)*);
            }

            pub(crate) fn create<'a, DB: Database + 'a>() -> Create<'a, DB>
                where
                    $(
                        $sql_type: Type<'a, DB>,
                    )*
                    $(
                        $sql_type2: Type<'a, DB>,
                    )*
            {
                Create::new(table_name)
                $(
                    .with_column(Column::new($table_column.name(), {
                        let ty: $sql_type = Default::default();
                        ty
                    }))
                )*
                $(
                    .with_column(Column::new($unmapped_column.name(), {
                        let ty: $sql_type2 = Default::default();
                        ty
                    }))
                )*
            }
        }
    };
}

macro_rules! __insertable {
    ($backend: ty, $model: ty, $($model_field: ident => $table_column: ident),*,) => {
        impl Insertable<$backend> for $model {
            fn table<'a>(&'a self) -> &'a Table {
                &table
            }

            fn values(&self) -> Vec<SetField<$backend>> {
                vec![
                    $(
                        $table_column.set(&self.$model_field)
                    ),*
                ]
            }
        }
    };
}