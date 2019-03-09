pub(crate) mod creator {
    use crate::core::backend::Error;
    use gdcf_model::user::Creator;

    use crate::schema::NowAtUtc;

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

pub(crate) mod profile {
    use crate::core::backend::Error;
    use gdcf_model::user::User;

    use crate::schema::NowAtUtc;

    use pm_gdcf_dbcache::{create, iqtable};

    iqtable! {
        User => profile {
            name => username,
            user_id => user_id,
            stars => stars,
            demons => demons,
            creator_points => creator_points,
            index_10 => index_10,
            index_11 => index_11,
            secret_coins => secret_coins,
            account_id => account_id,
            user_coins => user_coins,
            youtube_url => youtube_url,
            index_18 => index_18,
            index_19 => index_19,
            cube_index => cube_index,
            ship_index => ship_index,
            ball_index => ball_index,
            ufo_index => ufo_index,
            wave_index => wave_index,
            robot_index => robot_index,
            has_glow => has_glow,
            index_29 => index_29,
            global_rank => global_rank,
            index_31 => index_31,
            spider_index => spider_index,
            twitter_url => twitter_url,
            twitch_url => twitch_url,
            diamonds => diamonds,
            death_effect_index => death_effect_index,
            index_49 => index_49,
            index_50 => index_50,

            first_cached_at,
            last_cached_at
        }
    }

    create! {
        profile => {
            username: Text NotNull,
            user_id: Unsigned<BigInteger> NotNull Unique Primary,
            stars: Unsigned<Integer> NotNull,
            demons: Unsigned<SmallInteger> NotNull,
            creator_points: Unsigned<SmallInteger> NotNull,
            index_10: Text,
            index_11: Text,
            secret_coins: Unsigned<SmallInteger> NotNull,
            account_id: Unsigned<BigInteger> Unique,  // can be NULL!
            user_coins: Unsigned<SmallInteger> NotNull,
            index_18: Text,
            index_19: Text,
            youtube_url: Text,
            cube_index: Unsigned<SmallInteger> NotNull,
            ship_index: Unsigned<SmallInteger> NotNull,
            ball_index: Unsigned<SmallInteger> NotNull,
            ufo_index: Unsigned<SmallInteger> NotNull,
            wave_index: Unsigned<SmallInteger> NotNull,
            robot_index: Unsigned<SmallInteger> NotNull,
            has_glow: Boolean NotNull,
            index_29: Text,
            global_rank: Unsigned<Integer>, // can be NULL!
            index_31: Text,
            spider_index: Unsigned<SmallInteger> NotNull,
            twitter_url: Text,
            twitch_url: Text,
            diamonds: Unsigned<SmallInteger> NotNull,
            death_effect_index: Unsigned<SmallInteger>,
            index_49: Text,
            index_50: Text,

            first_cached_at: UtcTimestamp Default<NowAtUtc>(NowAtUtc) NotNull,
            last_cached_at: UtcTimestamp NotNull
        }
    }
}
