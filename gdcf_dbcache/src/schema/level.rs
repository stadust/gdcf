use gdcf::model::PartialLevel;

table! {
    PartialLevel => partial_level {
        level_id => level_id[Unsigned<BigInteger>][NotNull, Unique, PrimaryKey],
        name => level_name[Text][NotNull],
        description => description[Text],
        version => level_version[Unsigned<Integer>][NotNull],
        creator_id => creator_id[Unsigned<BigInteger>][NotNull],

        difficulty => difficulty[Text][NotNull],

        download => downloads[Unsigned<Integer>][NotNull],

        main_song => main_song[Unsigned<TinyInteger>],

        gd_version => gd_version[Text][NotNull],
        likes => likes[Integer][NotNull],

        length => level_length[Text][NotNull],
        stars => stars[Unsigned<TinyInteger>][NotNull],

        featured => featured[Integer][NotNull],

        copy_of => copy_of[Unsigned<Integer>],
        custom_song_id => custom_song_id[Unsigned<Integer>],
        coin_amount => coin_amount[Unsigned<TinyInteger>][NotNull],
        index_38 => index_38[Text],
        stars_requested => stars_requested[Unsigned<TinyInteger>],
        is_epic => is_epic[Boolean][NotNull],
        index_43 => index43[Text],
        object_amount => object_amount[Unsigned<Integer>][NotNull],
        index_46 => index_46[Text],
        index_47 => index_47[Text];
        first_cached_at[Timestamp][NotNull], last_cached_at[Timestamp][NotNull]
    }
}
