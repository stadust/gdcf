macro_rules! table {
    ($model: ident => $table: ident {$($model_field: ident => $table_column: ident),*; $($unmapped_column: ident),*}) => {
        pub(crate) mod $table {
            use super::{$model, $table};  // these import look weird (and they are, I dont get them), but they're needed
            use core::table::{Field, Table, SetField};

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
            use core::backend::pg::Pg;
            use core::query::insert::Insertable;

            #[cfg(feature = "pg")]
            impl Insertable<Pg> for $model {
                fn table<'a>(&'a self) -> &'a Table {
                    &$table::table
                }

                fn values(&self) -> Vec<SetField<Pg>> {
                    vec![
                        $(
                            $table::$table_column.set(&self.$model_field)
                        ),*
                    ]
                }
            }
        }
    };
}