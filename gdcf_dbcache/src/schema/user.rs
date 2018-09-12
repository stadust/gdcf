pub(crate) mod creator {
    use core::backend::Error;
    use gdcf::model::Creator;

    use schema::NowAtUtc;

    use pm_gdcf_dbcache::{create, iqtable};

    iqtable! {
        Creator => creator {
            user_id => user_id,
            name => user_name,
            account_id => account_id,

            first_cached_at,
            last_cached_at
        }
    }

    create! {
        creator => {
            user_id: Unsigned<BigInteger> NotNull Unique Primary,
            user_name: Text NotNull,
            account_id: Unsigned<BigInteger> Unique,
            first_cached_at: UtcTimestamp Default<NowAtUtc>(NowAtUtc) NotNull,
            last_cached_at: UtcTimestamp NotNull
        }
    }
}
