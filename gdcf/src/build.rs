use model::{Level, NewgroundsSong, PartialLevel};

pub(crate) fn build_partial_level(
    PartialLevel {
        level_id,
        name,
        description,
        version,
        creator_id,
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
    }: PartialLevel<u64>,
    custom_song: Option<NewgroundsSong>,
) -> PartialLevel<NewgroundsSong> {
    PartialLevel {
        custom_song,

        level_id,
        name,
        description,
        version,
        creator_id,
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

pub(crate) fn build_level(
    Level {
        base,
        level_data,
        password,
        time_since_update,
        time_since_upload,
        index_36,
    }: Level<u64>,
    song: Option<NewgroundsSong>,
) -> Level<NewgroundsSong> {
    trace!("Building a level with base {}", base);

    Level {
        base: build_partial_level(base, song),
        level_data,
        password,
        time_since_update,
        time_since_upload,
        index_36,
    }
}
