macro_rules! table {
    ($table_name: ident => {$($field: ident),*}) => {
        pub(crate) mod $table_name {
            use super::table::{Field, Table};

            #[allow(non_upper_case_globals)]
            const table_name: &str = stringify!($table_name);

            $(
                #[allow(non_upper_case_globals)]
                static $field: Field = Field {
                    table: table_name,
                    name: stringify!($field)
                };
            )*

            #[allow(non_upper_case_globals)]
            static $table_name: Table = Table {
                name: table_name,
                fields: &[
                    &$($field)*
                ]
            };
        }
    };
}