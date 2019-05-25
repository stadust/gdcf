CREATE TABLE newgrounds_song (
    song_id BIGINT PRIMARY KEY,
    song_name TEXT NOT NULL,
    index_3 INTEGER,
    song_artist TEXT NOT NULL,
    filesize DOUBLE PRECISION NOT NULL,
    index_6 TEXT,
    index_7 TEXT,
    index_8 TEXT,
    song_link TEXT
);

CREATE TABLE song_meta (
    song_id BIGINT PRIMARY KEY,
    cached_at TIMESTAMP WITHOUT TIME ZONE
);

CREATE TABLE creator (
    user_id BIGINT PRIMARY KEY,
    name TEXT NOT NULL,
    account_id BIGINT
);

CREATE TABLE creator_meta (
    user_id BIGINT PRIMARY KEY,
    cached_at TIMESTAMP WITHOUT TIME ZONE
);

CREATE TABLE partial_level (
    level_id BIGINT PRIMARY KEY,
    level_name TEXT NOT NULL,
    description TEXT,
    level_version INTEGER NOT NULL,
    creator_id BIGINT NOT NULL,
    difficulty TEXT NOT NULL,
    downloads INTEGER NOT NULL,
    main_song SMALLINT,
    gd_version SMALLINT NOT NULL,
    likes INTEGER NOT NULL,
    level_length TEXT NOT NULL,
    stars SMALLINT NOT NULL,
    featured INTEGER NOT NULL,
    copy_of BIGINT,
    custom_song_id BIGINT,
    coin_amount SMALLINT NOT NULL,
    coins_verified BOOLEAN NOT NULL,
    stars_requested SMALLINT,
    is_epic BOOLEAN NOT NULL,
    index_43 TEXT NOT NULL,
    object_amount INTEGER,
    index_46 TEXT,
    index_47 TEXT
);

CREATE TABLE partial_level_meta (
    level_id BIGINT PRIMARY KEY,
    cached_at TIMESTAMP WITHOUT TIME ZONE
);

CREATE TABLE level_list_meta (
    request_hash BIGINT PRIMARY KEY,
    cached_at TIMESTAMP WITHOUT TIME ZONE
);

CREATE TABLE request_results (
    level_id BIGINT NOT NULL,
    request_hash BIGINT NOT NULL
);

CREATE TABLE level (
    level_id BIGINT PRIMARY KEY REFERENCES partial_level (level_id),
    level_data BYTEA NOT NULL,
    level_password TEXT,
    time_since_upload TEXT NOT NULL,
    time_since_update TEXT NOT NULL,
    index_36 TEXT
);

CREATE TABLE level_meta (
    level_id BIGINT PRIMARY KEY,
    cached_at TIMESTAMP WITHOUT TIME ZONE
);

CREATE TABLE profile (
    username TEXT NOT NULL,
    user_id BIGINT PRIMARY KEY,
    stars INTEGER NOT NULL,
    demons SMALLINT NOT NULL,
    creator_points SMALLINT NOT NULL,
    index_10 TEXT,
    index_11 TEXT,
    secret_coins SMALLINT NOT NULL,
    account_id BIGINT NOT NULL,
    user_coins SMALLINT NOT NULL,
    index_18 TEXT,
    index_19 TEXT,
    youtube_url TEXT,
    cube_index SMALLINT NOT NULL,
    ship_index SMALLINT NOT NULL,
    ball_index SMALLINT NOT NULL,
    ufo_index SMALLINT NOT NULL,
    wave_index SMALLINT NOT NULL,
    robot_index SMALLINT NOT NULL,
    has_glow BOOLEAN NOT NULL,
    index_29 TEXT,
    global_rank INTEGER,
    index_31 TEXT,
    spider_index SMALLINT NOT NULL,
    twitter_url TEXT,
    twitch_url TEXT,
    diamonds SMALLINT NOT NULL,
    death_effect_index SMALLINT NOT NULL,
    index_49 TEXT,
    index_50 TEXT
);

CREATE TABLE profile_meta (
    user_id INTEGER PRIMARY KEY,
    cached_at TIMESTAMP WITHOUT TIME ZONE
);
