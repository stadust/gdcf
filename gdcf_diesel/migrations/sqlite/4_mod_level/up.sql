
PRAGMA foreign_keys=off;

ALTER TABLE profile RENAME TO temp_table;
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
    mod_level INTEGER NOT NULL,
    index_50 TEXT
);

INSERT INTO profile
    SELECT username, user_id, stars, demons, creator_points, index_10, index_11, secret_coins, account_id, user_coins, index_18, index_19, youtube_url, cube_index, ship_index, ball_index, ufo_index, wave_index, robot_index, has_glow, index_29, global_rank, index_31, spider_index, twitter_url, twitch_url, diamonds, death_effect_index, CAST(index_49 AS INTEGER) AS mod_level, index_50
    FROM temp_table;

DROP TABLE temp_table;

PRAGMA foreign_keys=on;