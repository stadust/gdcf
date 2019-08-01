
PRAGMA foreign_keys=off;

ALTER TABLE profile RENAME TO temp_table;
CREATE TABLE profile (
    username TEXT NOT NULL,
    user_id INTEGER PRIMARY KEY,
    stars INTEGER NOT NULL,
    demons INTEGER NOT NULL,
    creator_points INTEGER NOT NULL,
    index_10 INTEGER NOT NULL,
    index_11 INTEGER NOT NULL,
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
    SELECT username, user_id, stars, demons, creator_points,
    CASE
        WHEN primary_color = 125 | (255 << 8) | (0 << 16) THEN "0"
        WHEN primary_color = 0 | (255 << 8) | (0 << 16) THEN "1"
        WHEN primary_color = 0 | (255 << 8) | (125 << 16) THEN "2"
        WHEN primary_color = 0 | (255 << 8) | (255 << 16) THEN "3"
        WHEN primary_color = 0 | (200 << 8) | (255 << 16) THEN "16"
        WHEN primary_color = 0 | (125 << 8) | (255 << 16) THEN "4"
        WHEN primary_color = 0 | (0 << 8) | (255 << 16) THEN "5"
        WHEN primary_color = 125 | (0 << 8) | (255 << 16) THEN "6"
        WHEN primary_color = 185 | (0 << 8) | (255 << 16) THEN "13"
        WHEN primary_color = 255 | (0 << 8) | (255 << 16) THEN "7"
        WHEN primary_color = 255 | (0 << 8) | (125 << 16) THEN "8"
        WHEN primary_color = 255 | (0 << 8) | (0 << 16) THEN "9"
        WHEN primary_color = 255 | (75 << 8) | (0 << 16) THEN "29"
        WHEN primary_color = 255 | (125 << 8) | (0 << 16) THEN "10"
        WHEN primary_color = 255 | (185 << 8) | (0 << 16) THEN "14"
        WHEN primary_color = 255 | (255 << 8) | (0 << 16) THEN "11"
        WHEN primary_color = 255 | (255 << 8) | (255 << 16) THEN "12"
        WHEN primary_color = 175 | (175 << 8) | (175 << 16) THEN "17"
        WHEN primary_color = 80 | (80 << 8) | (80 << 16) THEN "18"
        WHEN primary_color = 0 | (0 << 8) | (0 << 16) THEN "15"
        WHEN primary_color = 125 | (125 << 8) | (0 << 16) THEN "27"
        WHEN primary_color = 100 | (150 << 8) | (0 << 16) THEN "32"
        WHEN primary_color = 75 | (175 << 8) | (0 << 16) THEN "28"
        WHEN primary_color = 0 | (150 << 8) | (0 << 16) THEN "38"
        WHEN primary_color = 0 | (175 << 8) | (75 << 16) THEN "20"
        WHEN primary_color = 0 | (150 << 8) | (100 << 16) THEN "33"
        WHEN primary_color = 0 | (125 << 8) | (125 << 16) THEN "21"
        WHEN primary_color = 0 | (100 << 8) | (150 << 16) THEN "34"
        WHEN primary_color = 0 | (75 << 8) | (175 << 16) THEN "22"
        WHEN primary_color = 0 | (0 << 8) | (150 << 16) THEN "39"
        WHEN primary_color = 75 | (0 << 8) | (175 << 16) THEN "23"
        WHEN primary_color = 100 | (0 << 8) | (150 << 16) THEN "35"
        WHEN primary_color = 125 | (0 << 8) | (125 << 16) THEN "24"
        WHEN primary_color = 150 | (0 << 8) | (100 << 16) THEN "36"
        WHEN primary_color = 175 | (0 << 8) | (75 << 16) THEN "25"
        WHEN primary_color = 150 | (0 << 8) | (0 << 16) THEN "37"
        WHEN primary_color = 150 | (50 << 8) | (0 << 16) THEN "30"
        WHEN primary_color = 175 | (75 << 8) | (0 << 16) THEN "26"
        WHEN primary_color = 150 | (100 << 8) | (0 << 16) THEN "31"
        WHEN primary_color = 255 | (255 << 8) | (125 << 16) THEN "19"
        WHEN primary_color = 125 | (255 << 8) | (175 << 16) THEN "40"
        WHEN primary_color = 125 | (125 << 8) | (255 << 16) THEN "41"
        ELSE CAST(-primary_color AS TEXT)
    END index_10,
    CASE
        WHEN secondary_color = 125 | (255 << 8) | (0 << 16) THEN "0"
        WHEN secondary_color = 0 | (255 << 8) | (0 << 16) THEN "1"
        WHEN secondary_color = 0 | (255 << 8) | (125 << 16) THEN "2"
        WHEN secondary_color = 0 | (255 << 8) | (255 << 16) THEN "3"
        WHEN secondary_color = 0 | (200 << 8) | (255 << 16) THEN "16"
        WHEN secondary_color = 0 | (125 << 8) | (255 << 16) THEN "4"
        WHEN secondary_color = 0 | (0 << 8) | (255 << 16) THEN "5"
        WHEN secondary_color = 125 | (0 << 8) | (255 << 16) THEN "6"
        WHEN secondary_color = 185 | (0 << 8) | (255 << 16) THEN "13"
        WHEN secondary_color = 255 | (0 << 8) | (255 << 16) THEN "7"
        WHEN secondary_color = 255 | (0 << 8) | (125 << 16) THEN "8"
        WHEN secondary_color = 255 | (0 << 8) | (0 << 16) THEN "9"
        WHEN secondary_color = 255 | (75 << 8) | (0 << 16) THEN "29"
        WHEN secondary_color = 255 | (125 << 8) | (0 << 16) THEN "10"
        WHEN secondary_color = 255 | (185 << 8) | (0 << 16) THEN "14"
        WHEN secondary_color = 255 | (255 << 8) | (0 << 16) THEN "11"
        WHEN secondary_color = 255 | (255 << 8) | (255 << 16) THEN "12"
        WHEN secondary_color = 175 | (175 << 8) | (175 << 16) THEN "17"
        WHEN secondary_color = 80 | (80 << 8) | (80 << 16) THEN "18"
        WHEN secondary_color = 0 | (0 << 8) | (0 << 16) THEN "15"
        WHEN secondary_color = 125 | (125 << 8) | (0 << 16) THEN "27"
        WHEN secondary_color = 100 | (150 << 8) | (0 << 16) THEN "32"
        WHEN secondary_color = 75 | (175 << 8) | (0 << 16) THEN "28"
        WHEN secondary_color = 0 | (150 << 8) | (0 << 16) THEN "38"
        WHEN secondary_color = 0 | (175 << 8) | (75 << 16) THEN "20"
        WHEN secondary_color = 0 | (150 << 8) | (100 << 16) THEN "33"
        WHEN secondary_color = 0 | (125 << 8) | (125 << 16) THEN "21"
        WHEN secondary_color = 0 | (100 << 8) | (150 << 16) THEN "34"
        WHEN secondary_color = 0 | (75 << 8) | (175 << 16) THEN "22"
        WHEN secondary_color = 0 | (0 << 8) | (150 << 16) THEN "39"
        WHEN secondary_color = 75 | (0 << 8) | (175 << 16) THEN "23"
        WHEN secondary_color = 100 | (0 << 8) | (150 << 16) THEN "35"
        WHEN secondary_color = 125 | (0 << 8) | (125 << 16) THEN "24"
        WHEN secondary_color = 150 | (0 << 8) | (100 << 16) THEN "36"
        WHEN secondary_color = 175 | (0 << 8) | (75 << 16) THEN "25"
        WHEN secondary_color = 150 | (0 << 8) | (0 << 16) THEN "37"
        WHEN secondary_color = 150 | (50 << 8) | (0 << 16) THEN "30"
        WHEN secondary_color = 175 | (75 << 8) | (0 << 16) THEN "26"
        WHEN secondary_color = 150 | (100 << 8) | (0 << 16) THEN "31"
        WHEN secondary_color = 255 | (255 << 8) | (125 << 16) THEN "19"
        WHEN secondary_color = 125 | (255 << 8) | (175 << 16) THEN "40"
        WHEN secondary_color = 125 | (125 << 8) | (255 << 16) THEN "41"
        ELSE CAST(-secondary_color AS TEXT)
    END index_11, secret_coins, account_id, user_coins, index_18, index_19, youtube_url, cube_index, ship_index, ball_index, ufo_index, wave_index, robot_index, has_glow, index_29, global_rank, index_31, spider_index, twitter_url, twitch_url, diamonds, death_effect_index, mod_level, index_50
    FROM temp_table;

DROP TABLE temp_table;

PRAGMA foreign_keys=on;