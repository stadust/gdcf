pub(crate) mod partial_level {
    use crate::core::backend::Error;
    use gdcf_model::level::PartialLevel;

    use crate::schema::NowAtUtc;

    use pm_gdcf_dbcache::{create, iqtable};

    iqtable! {
        PartialLevel<u64, u64> => partial_level {
            level_id => level_id,
            name => level_name,
            description => description,
            version => level_version,
            creator => creator_id,

            difficulty => difficulty,

            downloads => downloads,

            main_song => main_song,

            gd_version => gd_version,
            likes => likes,

            length => level_length,
            stars => stars,

            featured => featured,

            copy_of => copy_of,
            custom_song => custom_song_id,
            coin_amount => coin_amount,
            coins_verified => coins_verified,
            stars_requested => stars_requested,
            is_epic => is_epic,
            index_43 => index_43,
            object_amount => object_amount,
            index_46 => index_46,
            index_47 => index_47,
            first_cached_at,
            last_cached_at
        }
    }

    create! {
        partial_level => {
            level_id: Unsigned<BigInteger> NotNull Unique Primary,
            level_name: Text NotNull,
            description: Text,
            level_version: Unsigned<Integer> NotNull,
            creator_id: Unsigned<BigInteger> NotNull,

            difficulty: Text NotNull,

            downloads: Unsigned<Integer> NotNull,

            main_song: Unsigned<SmallInteger>,

            gd_version: Unsigned<SmallInteger> NotNull,

            likes: Integer NotNull,

            level_length: Text NotNull,

            stars: Unsigned<SmallInteger> NotNull,

            featured: Integer NotNull,

            copy_of: Unsigned<BigInteger>,
            custom_song_id: Unsigned<BigInteger>,
            coin_amount: Unsigned<SmallInteger> NotNull,
            coins_verified: Boolean,
            stars_requested: Unsigned<SmallInteger>,
            is_epic: Boolean NotNull,
            index_43: Text,
            object_amount: Unsigned<Integer> NotNull,
            index_46: Text,
            index_47: Text,

            first_cached_at: UtcTimestamp Default<NowAtUtc>(NowAtUtc) NotNull,
            last_cached_at: UtcTimestamp NotNull
        }
    }
}

pub(crate) mod partial_levels {
    use pm_gdcf_dbcache::{create, table};

    table! {
        _ => partial_levels {
            level_id,
            request_hash,
            first_cached_at,
            last_cached_at
        }
    }

    create! {
        partial_levels => {
            level_id: Unsigned<BigInteger>,
            request_hash: Unsigned<BigInteger>
        }
    }

    pub(crate) mod cached_at {
        use crate::schema::NowAtUtc;

        use pm_gdcf_dbcache::{create, table};

        table! {
            _ => partial_levels_request_cached_at {
                request_hash,
                first_cached_at,
                last_cached_at
            }
        }

        create! {
            partial_levels_request_cached_at => {
                request_hash: Unsigned<BigInteger> Unique NotNull Primary,

                first_cached_at: UtcTimestamp Default<NowAtUtc>(NowAtUtc) NotNull,
                last_cached_at: UtcTimestamp NotNull
            }
        }
    }
}

pub(crate) mod full_level {
    use pm_gdcf_dbcache::{create, itable};

    use crate::{
        core::{
            backend::Error,
            query::{
                select::{Queryable, Row},
                Select,
            },
        },
        schema::NowAtUtc,
    };
    use gdcf_model::level::{Level, PartialLevel};

    itable! {
        Level<u64, u64> => level {
            level_id,
            level_data => level_data,
            password => level_password,
            time_since_upload => time_since_upload,
            time_since_update => time_since_update,
            index_36 => index_36,

            first_cached_at,
            last_cached_at
        }
    }

    create! {
        level => {
            level_id: Unsigned<BigInteger> NotNull Unique Primary,
            level_data: Bytes NotNull,
            level_password: Text,
            time_since_upload: Text,
            time_since_update: Text,
            index_36: Text,

            first_cached_at: UtcTimestamp Default<NowAtUtc>(NowAtUtc) NotNull,
            last_cached_at: UtcTimestamp NotNull
        }
    }

    // TODO: the other backends
    // TODO: better way of doing this
    #[cfg(feature = "pg")]
    use crate::core::backend::pg::Pg;
    #[cfg(feature = "pg")]
    impl Queryable<Pg> for Level<u64, u64> {
        fn select_from(from: Table) -> Select<Pg> {
            Select::new(from, Vec::new())
                .join(super::partial_level::table, level_id.same_as(super::partial_level::level_id))
                .select(&super::partial_level::table.fields()[..24])
                .select(&from.fields()[1..])
        }

        fn from_row(row: &Row<Pg>, offset: isize) -> Result<Self, Error<Pg>> {
            let base = PartialLevel::from_row(row, offset)?;

            Ok(Level {
                base,
                level_data: row.get(offset + 24).unwrap()?,
                password: row.get(offset + 25).unwrap()?,
                time_since_upload: row.get(offset + 26).unwrap()?,
                time_since_update: row.get(offset + 27).unwrap()?,
                index_36: row.get(offset + 28).unwrap()?,
            })
        }
    }
    #[cfg(feature = "sqlite")]
    use crate::core::backend::sqlite::Sqlite;
    #[cfg(feature = "sqlite")]
    impl Queryable<Sqlite> for Level<u64, u64> {
        fn select_from(from: Table) -> Select<Sqlite> {
            Select::new(from, Vec::new())
                .join(super::partial_level::table, level_id.same_as(super::partial_level::level_id))
                .select(&super::partial_level::table.fields()[..24])
                .select(&from.fields()[1..])
        }

        fn from_row(row: &Row<Sqlite>, offset: isize) -> Result<Self, Error<Sqlite>> {
            let base = PartialLevel::from_row(row, offset)?;

            Ok(Level {
                base,
                level_data: row.get(offset + 24).unwrap()?,
                password: row.get(offset + 25).unwrap()?,
                time_since_upload: row.get(offset + 26).unwrap()?,
                time_since_update: row.get(offset + 27).unwrap()?,
                index_36: row.get(offset + 28).unwrap()?,
            })
        }
    }

}
