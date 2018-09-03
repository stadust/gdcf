use cache::Cache;
use error::CacheError;
use model::{Level, NewgroundsSong, PartialLevel};

pub(crate) fn build_partial_level<C: Cache>(
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
        custom_song: custom_song_id,
        coin_amount,
        coins_verified,
        stars_requested,
        is_epic,
        index_43,
        object_amount,
        index_46,
        index_47,
    }: PartialLevel<u64>,
    cache: &C,
) -> Result<PartialLevel<NewgroundsSong>, CacheError<C::Err>> {
    let custom_song = match custom_song_id {
        None => None,
        Some(song_id) => Some(cache.lookup_song(song_id)?.extract()),
    };

    Ok(PartialLevel {
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
    })
}

pub(crate) fn build_level<C: Cache>(
    Level {
        base,
        level_data,
        password,
        time_since_update,
        time_since_upload,
        index_36,
    }: Level<u64>,
    cache: &C,
) -> Result<Level<NewgroundsSong>, CacheError<C::Err>> {
    Ok(Level {
        base: build_partial_level(base, cache)?,
        level_data,
        password,
        time_since_update,
        time_since_upload,
        index_36,
    })
}
