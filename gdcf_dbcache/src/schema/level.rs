use gdcf::model::PartialLevel;

table! {
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
        index_47 => index_47;
        first_cached_at, last_cached_at
    }
}

create! { partial_level,
    level_id[NotNull, Unique, Primary] => Unsigned<BigInteger>,
    level_name[NotNull] => Text,
    description => Text,
    level_version[NotNull] => Unsigned<Integer>,
    creator_id[NotNull] => Unsigned<BigInteger>,

    difficulty[NotNull] => Text,

    downloads[NotNull] => Unsigned<Integer>,

    main_song => Unsigned<SmallInteger>,

    gd_version[NotNull] => Text,

    likes[NotNull] => Integer,

    level_length[NotNull] => Text,

    stars[NotNull] => Unsigned<SmallInteger>,

    featured[NotNull] => Integer,

    copy_of => Unsigned<BigInteger>,
    custom_song_id => Unsigned<BigInteger>,
    coin_amount[NotNull] => Unsigned<SmallInteger>,
    index_38 => Text,
    stars_requested => Unsigned<SmallInteger>,
    is_epic[NotNull] => Boolean,
    index_43 => Text,
    object_amount[NotNull] => Unsigned<Integer>,
    index_46 => Text,
    index_47 => Text,
    first_cached_at[NotNull] => UtcTimestamp,
    last_cached_at[NotNull] => UtcTimestamp
}