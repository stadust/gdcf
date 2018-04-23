CREATE TABLE newgrounds_song (
    song_id BIGINT PRIMARY KEY NOT NULL,
    song_name TEXT NOT NULL,
    artist TEXT NOT NULL,
    filesize DOUBLE PRECISION NOT NULL,
    alt_artist TEXT NULL DEFAULT NULL,
    banned BOOLEAN NOT NULL DEFAULT false,
    download_link TEXT NOT NULL,
    internal_id BIGINT UNIQUE NOT NULL,
    first_cached_at TIMESTAMP WITHOUT TIME ZONE NOT NULL DEFAULT (now() AT TIME ZONE 'utc'),
    last_cached_at TIMESTAMP WITHOUT TIME ZONE NOT NULL DEFAULT (now() AT TIME ZONE 'utc')
);

CREATE TABLE partial_level (
    level_id BIGINT PRIMARY KEY NOT NULL,
    level_name TEXT NOT NULL,
    description TEXT NOT NULL,
    level_version INT NOT NULL,
    creator_id BIGINT NOT NULL,
    -- omitted: has_difficulty_rating
    -- TODO: difficulty
    downloads BIGINT NOT NULL,
    main_song SMALLINT NULL DEFAULT NULL,
    -- TODO: length
    -- omitted: is_demon (should be included in difficulty)
    stars SMALLINT NOT NULL,
    featured_weight INTEGER NOT NULL,
    -- omitted: is_auto (should be included in difficulty)
    copy_of BIGINT NULL DEFAULT NULL,
    custom_song_id BIGINT NULL DEFAULT NULL,
    coins SMALLINT NOT NULL,
    stars_requested SMALLINT NULL DEFAULT NULL,
    is_epic BOOL NOT NULL,
    object_amount INTEGER NOT NULL,
    first_cached_at TIMESTAMP WITHOUT TIME ZONE NOT NULL DEFAULT (now() AT TIME ZONE 'utc'),
    last_cached_at TIMESTAMP WITHOUT TIME ZONE NOT NULL DEFAULT (now() AT TIME ZONE 'utc')
);

CREATE TABLE level (
    base BIGINT REFERENCES partial_level (level_id),
    level_data TEXT NOT NULL,
    -- TODO: password
    -- TODO: time_since_upload
    -- TODO: time_since_update
    first_cached_at TIMESTAMP WITHOUT TIME ZONE NOT NULL DEFAULT (now() AT TIME ZONE 'utc'),
    last_cached_at TIMESTAMP WITHOUT TIME ZONE NOT NULL DEFAULT (now() AT TIME ZONE 'utc')
)

CREATE TABLE partial_level_search_result (
    request TEXT NOT NULL UNIQUE,
    result_index SMALLINT NOT NULL,
    result_level BIGINT REFERENCES partial_level (level_id),
    first_cached_at TIMESTAMP WITHOUT TIME ZONE NOT NULL DEFAULT (now() AT TIME ZONE 'utc'),
    last_cached_at TIMESTAMP WITHOUT TIME ZONE NOT NULL DEFAULT (now() AT TIME ZONE 'utc')
)
