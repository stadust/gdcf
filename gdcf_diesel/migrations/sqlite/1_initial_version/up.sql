CREATE TABLE newgrounds_song (
    song_id INTEGER PRIMARY KEY,
    song_name TEXT NOT NULL,
    index_3 INTEGER,
    song_artist TEXT NOT NULL,
    filesize REAL NOT NULL,
    index_6 TEXT,
    index_7 TEXT,
    index_8 TEXT,
    song_link TEXT
);

CREATE TABLE song_meta (
    song_id INTEGER PRIMARY KEY,
    cached_at INTEGER
);

CREATE TABLE creator (
    user_id INTEGER PRIMARY KEY,
    name TEXT NOT NULL,
    account_id INTEGER
);

CREATE TABLE creator_meta (
    user_id INTEGER PRIMARY KEY,
    cached_at INTEGER
);

CREATE TABLE partial_level (
    level_id INTEGER PRIMARY KEY,
    level_name TEXT NOT NULL,
    description TEXT,
    level_version INTEGER NOT NULL,
    creator_id INTEGER NOT NULL,
    difficulty TEXT NOT NULL,
    downloads INTEGER NOT NULL,
    main_song INTEGER,
    gd_version INTEGER NOT NULL,
    likes INTEGER NOT NULL,
    level_length TEXT NOT NULL,
    stars INTEGER NOT NULL,
    featured INTEGER NOT NULL,
    copy_of INTEGER,
    custom_song_id INTEGER,
    coin_amount INTEGER NOT NULL,
    coins_verified BOOLEAN NOT NULL,
    stars_requested INTEGER,
    is_epic BOOLEAN NOT NULL,
    index_43 TEXT NOT NULL,
    object_amount INTEGER,
    index_46 TEXT,
    index_47 TEXT
);

CREATE TABLE partial_level_meta (
    level_id INTEGER PRIMARY KEY,
    cached_at INTEGER
);

CREATE TABLE level_list_meta (
    request_hash INTEGER PRIMARY KEY,
    cached_at INTEGER
);

CREATE TABLE request_results (
    level_id INTEGER NOT NULL,
    request_hash INTEGER NOT NULL
);

CREATE TABLE level (
    level_id INTEGER PRIMARY KEY,
    level_data BLOB NOT NULL,
    level_password TEXT,
    time_since_upload TEXT NOT NULL,
    time_since_update TEXT NOT NULL,
    index_36 TEXT,
    FOREIGN KEY (level_id) REFERENCES partial_level(level_id)
);

CREATE TABLE level_meta (
    level_id INTEGER PRIMARY KEY,
    cached_at INTEGER
);

CREATE TABLE profile (
    username TEXT NOT NULL,
    user_id INTEGER PRIMARY KEY,
    stars INTEGER NOT NULL,
    demons INTEGER NOT NULL,
    creator_points INTEGER NOT NULL,
    index_10 TEXT,
    index_11 TEXT,
    secret_coins INTEGER NOT NULL,
    account_id INTEGER NOT NULL,
    user_coins INTEGER NOT NULL,
    index_18 TEXT,
    index_19 TEXT,
    youtube_url TEXT,
    cube_index INTEGER NOT NULL,
    ship_index INTEGER NOT NULL,
    ball_index INTEGER NOT NULL,
    ufo_index INTEGER NOT NULL,
    wave_index INTEGER NOT NULL,
    robot_index INTEGER NOT NULL,
    has_glow BOOLEAN NOT NULL,
    index_29 TEXT,
    global_rank INTEGER,
    index_31 TEXT,
    spider_index INTEGER NOT NULL,
    twitter_url TEXT,
    twitch_url TEXT,
    diamonds INTEGER NOT NULL,
    death_effect_index INTEGER NOT NULL,
    index_49 TEXT,
    index_50 TEXT
);

CREATE TABLE profile_meta (
    user_id INTEGER PRIMARY KEY,
    cached_at INTEGER
);