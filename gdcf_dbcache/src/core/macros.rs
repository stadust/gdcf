macro_rules! if_query_part {
    ($t: ty, $tr: ty) => {
        impl<'a, DB: Database> $tr for $t where $t: QueryPart<DB> {}
    };
}

macro_rules! simple_query_part {
    ($back: ty, $t: ty, $val: expr) => {
        impl QueryPart<$back> for $t {
            fn to_sql(&self) -> Preparation<$back> {
                Preparation::<$back>::default().with_static($val)
            }
        }
    };
}

macro_rules! as_sql_cast {
    ($back: ty, $src: ty, $dest: ty, $variant: path) => {
        impl AsSql<$back> for $src {
            fn as_sql(&self) -> <$back as Database>::Types {
                $variant(*self as $dest)
            }
        }
    };
}

macro_rules! as_sql_cast_lossless {
    ($back: ty, $src: ty, $dest: ident, $variant: path) => {
        impl AsSql<$back> for $src {
            fn as_sql(&self) -> <$back as Database>::Types {
                $variant($dest::from(*self))
            }
        }
    };
}

macro_rules! from_sql_cast {
    ($back: ty, $dest: ty, $variant: path) => {
        impl FromSql<$back> for $dest {
            fn from_sql(sql: &<$back as Database>::Types) -> Result<Self, Error<$back>> {
                match sql {
                    $variant(value) => Ok(*value as $dest),
                    _ => Err(Error::Conversion(format!("{:?}", sql), stringify!($dest))),
                }
            }
        }
    };
}
