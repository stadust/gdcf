macro_rules! table {
    ($table_name: ident => {$($field: ident),*}) => {
        pub(crate) mod $table_name {
            use core::table::{Field, Table};

            #[allow(non_upper_case_globals)]
            pub(crate) const table_name: &str = stringify!($table_name);

            $(
                #[allow(non_upper_case_globals)]
                pub(crate) static $field: Field = Field {
                    table: table_name,
                    name: stringify!($field)
                };
            )*

            #[allow(non_upper_case_globals)]
            pub(crate) static table: Table = Table {
                name: table_name,
                fields: &[
                    $(&$field,)*
                ]
            };
        }
    };
}

macro_rules! insertable {
    ($model: ty => $table: ident {$($model_field: ident => $table_column: ident),*}) => {
        #[cfg(feature = "pg")]
        use core::backend::pg::Pg;
        use core::query::insert::Insertable;
        use core::table::{Table, SetField};

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
    };
}