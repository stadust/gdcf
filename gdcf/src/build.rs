use model::{Level, NewgroundsSong, PartialLevel};

pub(crate) fn partial_level_song<User: PartialEq>(
    PartialLevel {
        level_id,
        name,
        description,
        version,
        creator,
        difficulty,
        downloads,
        main_song,
        gd_version,
        likes,
        length,
        stars,
        featured,
        copy_of,
        coin_amount,
        coins_verified,
        stars_requested,
        is_epic,
        index_43,
        object_amount,
        index_46,
        index_47,
        ..
    }: PartialLevel<u64, User>,
    custom_song: Option<NewgroundsSong>,
) -> PartialLevel<NewgroundsSong, User> {
    PartialLevel {
        custom_song,

        level_id,
        name,
        description,
        version,
        creator,
        difficulty,
        downloads,
        main_song,
        gd_version,
        likes,
        length,
        stars,
        featured,
        copy_of,
        coin_amount,
        coins_verified,
        stars_requested,
        is_epic,
        index_43,
        object_amount,
        index_46,
        index_47,
    }
}

pub(crate) fn level_song<User: PartialEq>(
    Level {
        base,
        level_data,
        password,
        time_since_update,
        time_since_upload,
        index_36,
    }: Level<u64, User>,
    song: Option<NewgroundsSong>,
) -> Level<NewgroundsSong, User> {
    trace!("Building a level with base {}", base);

    Level {
        base: partial_level_song(base, song),
        level_data,
        password,
        time_since_update,
        time_since_upload,
        index_36,
    }
}
