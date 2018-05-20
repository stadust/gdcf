use core::backend::pg::Pg;
use core::query::QueryPart;
use core::types::{BigInteger, Boolean, Double, Float, Integer, Text, TinyInteger, Unsigned, UtcTimestamp};

simply_query_part!(Pg, Text, "TEXT");
simply_query_part!(Pg, TinyInteger, "TINYINT");
simply_query_part!(Pg, Integer, "INT");
simply_query_part!(Pg, BigInteger, "BIGINT");
simply_query_part!(Pg, Boolean, "BOOL");
simply_query_part!(Pg, Float, "FLOAT(4)");
simply_query_part!(Pg, Double, "REAL");
simply_query_part!(Pg, Unsigned<Integer>, "INTEGER");
simply_query_part!(Pg, Unsigned<BigInteger>, "BIGINT");
simply_query_part!(Pg, UtcTimestamp, "TIMESTAMP WITHOUT TIMEZONE");