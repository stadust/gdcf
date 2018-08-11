macro_rules! if_query_part {
    ($t: ty, $tr: ty) => {
        impl<'a, DB: Database> $tr for $t
            where
                $t: QueryPart<DB>
        {}
    };
}

macro_rules! simple_query_part {
    ($back: ty, $t: ty, $val: expr) => {
        impl QueryPart<$back> for $t {
            fn to_sql_unprepared(&self) -> String {
                String::from($val)
            }
        }
    };
}