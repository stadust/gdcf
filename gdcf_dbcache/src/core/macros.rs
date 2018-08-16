macro_rules! if_query_part {
    ($t: ty, $tr: ty) => {
        impl<'a, DB: Database> $tr for $t
            where
                $t: QueryPart<DB>
        {}
    };
}

macro_rules! if_sql_expr {
    ($t: ty, $tr: ty) => {
        impl<'a, DB: Database> $tr for $t
            where
                $t: SqlExpr<DB>
        {}
    }
}

macro_rules! simple_query_part {
    ($back: ty, $t: ty, $val: expr) => {
        impl QueryPart<$back> for $t {
            /*fn to_sql_unprepared(&self) -> String {
                String::from($val)
            }*/
            fn to_sql(&self) -> Preparation<$back> {
                Preparation::<$back>::default()
                    .with_static($val)
            }
        }
    };
}

macro_rules! as_sql {
    ($backend: ty, $target: ty => $conversion: expr) => {
        impl AsSql<$backend> for $target {
            fn as_sql(&self) -> <$backend as Database>::Types {
                $conversion
            }
        }

        impl QueryPart<$backend> for $target {
            fn to_sql(&self) -> Preparation<$backend> {
                (PreparedStatement::default(), vec![self])
            }

            fn to_raw_sql(&self) -> String {
                self.as_sql().to_string()
            }
        }
    }
}