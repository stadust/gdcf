pub(crate) mod partial_level {
    use gdcf::model::PartialLevel;

    use core::backend::Error;

    use schema::NowAtUtc;

    use pm_gdcf_dbcache::{iqtable, create};

    iqtable! {
        PartialLevel => partial_level {
            level_id => level_id,
            name => level_name,
            description => description,
            version => level_version,
            creator_id => creator_id,

            difficulty => difficulty,

            downloads => downloads,

            main_song => main_song,

            gd_version => gd_version,
            likes => likes,

            length => level_length,
            stars => stars,

            featured => featured,

            copy_of => copy_of,
            custom_song_id => custom_song_id,
            coin_amount => coin_amount,
            index_38 => index_38,
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
            index_38: Text,
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
    //use core::query::select::Select;
    //use gdcf::api::request::LevelsRequest;

    use pm_gdcf_dbcache::{table, create};

    //use util;

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
        use schema::NowAtUtc;

        use pm_gdcf_dbcache::{table, create};

        table! {
            _ => partial_levels_request_cached_at {
                request_hash,
                first_cached_at,
                last_cached_at
            }
        }

        create! {
            partial_levels_request_cached_at => {
                request_hash: Unsigned<BigInteger> Unique NotNull,

                first_cached_at: UtcTimestamp Default<NowAtUtc>(NowAtUtc) NotNull,
                last_cached_at: UtcTimestamp NotNull
            }
        }
    }

    /*use core::query::condition::*;
    use core::*;

    pub fn lookup<'a, DB: Database + 'a>(req: &LevelsRequest) -> Select<'a, DB>
        where
            And<DB>: Condition<DB>,
            EqValue<'a, DB>: Condition<DB> + 'static,
            u32: AsSql<DB>,
            u64: AsSql<DB>
    {
        let req_hash = util::hash(req);

        Select::new(&table, Vec::new())
            .filter(request_hash.eq(req_hash))
            // TODO: join partial_level and only return those things.
    }*/
}