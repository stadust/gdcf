use core::backend::pg::Pg;
use core::QueryPart;
use core::statement::Preparation;
use core::statement::Prepare;
use core::types::{BigInteger, Boolean, Bytes, Double, Float, Integer, SmallInteger, Text, Unsigned, UtcTimestamp};

simple_query_part!(Pg, Text, "TEXT");
simple_query_part!(Pg, SmallInteger, "SMALLINT");
simple_query_part!(Pg, Integer, "INT");
simple_query_part!(Pg, BigInteger, "BIGINT");
simple_query_part!(Pg, Boolean, "BOOL");
simple_query_part!(Pg, Float, "FLOAT(4)");
simple_query_part!(Pg, Double, "DOUBLE PRECISION");
simple_query_part!(Pg, Unsigned<SmallInteger>, "SMALLINT");
simple_query_part!(Pg, Unsigned<Integer>, "INTEGER");
simple_query_part!(Pg, Unsigned<BigInteger>, "BIGINT");
simple_query_part!(Pg, UtcTimestamp, "TIMESTAMP WITHOUT TIME ZONE");
simple_query_part!(Pg, Bytes, "BYTEA");