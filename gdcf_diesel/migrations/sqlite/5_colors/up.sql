
PRAGMA foreign_keys=off;

ALTER TABLE profile RENAME TO temp_table;
CREATE TABLE profile (
    username TEXT NOT NULL,
    user_id INTEGER PRIMARY KEY,
    stars INTEGER NOT NULL,
    demons INTEGER NOT NULL,
    creator_points INTEGER NOT NULL,
    primary_color INTEGER NOT NULL,
    secondary_color INTEGER NOT NULL,
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
        WHEN index_10 = "0" THEN 125 | (255 << 8) | (0 << 16)
        WHEN index_10 = "1" THEN 0 | (255 << 8) | (0 << 16)
        WHEN index_10 = "2" THEN 0 | (255 << 8) | (125 << 16)
        WHEN index_10 = "3" THEN 0 | (255 << 8) | (255 << 16)
        WHEN index_10 = "16" THEN 0 | (200 << 8) | (255 << 16)
        WHEN index_10 = "4" THEN 0 | (125 << 8) | (255 << 16)
        WHEN index_10 = "5" THEN 0 | (0 << 8) | (255 << 16)
        WHEN index_10 = "6" THEN 125 | (0 << 8) | (255 << 16)
        WHEN index_10 = "13" THEN 185 | (0 << 8) | (255 << 16)
        WHEN index_10 = "7" THEN 255 | (0 << 8) | (255 << 16)
        WHEN index_10 = "8" THEN 255 | (0 << 8) | (125 << 16)
        WHEN index_10 = "9" THEN 255 | (0 << 8) | (0 << 16)
        WHEN index_10 = "29" THEN 255 | (75 << 8) | (0 << 16)
        WHEN index_10 = "10" THEN 255 | (125 << 8) | (0 << 16)
        WHEN index_10 = "14" THEN 255 | (185 << 8) | (0 << 16)
        WHEN index_10 = "11" THEN 255 | (255 << 8) | (0 << 16)
        WHEN index_10 = "12" THEN 255 | (255 << 8) | (255 << 16)
        WHEN index_10 = "17" THEN 175 | (175 << 8) | (175 << 16)
        WHEN index_10 = "18" THEN 80 | (80 << 8) | (80 << 16)
        WHEN index_10 = "15" THEN 0 | (0 << 8) | (0 << 16)
        WHEN index_10 = "27" THEN 125 | (125 << 8) | (0 << 16)
        WHEN index_10 = "32" THEN 100 | (150 << 8) | (0 << 16)
        WHEN index_10 = "28" THEN 75 | (175 << 8) | (0 << 16)
        WHEN index_10 = "38" THEN 0 | (150 << 8) | (0 << 16)
        WHEN index_10 = "20" THEN 0 | (175 << 8) | (75 << 16)
        WHEN index_10 = "33" THEN 0 | (150 << 8) | (100 << 16)
        WHEN index_10 = "21" THEN 0 | (125 << 8) | (125 << 16)
        WHEN index_10 = "34" THEN 0 | (100 << 8) | (150 << 16)
        WHEN index_10 = "22" THEN 0 | (75 << 8) | (175 << 16)
        WHEN index_10 = "39" THEN 0 | (0 << 8) | (150 << 16)
        WHEN index_10 = "23" THEN 75 | (0 << 8) | (175 << 16)
        WHEN index_10 = "35" THEN 100 | (0 << 8) | (150 << 16)
        WHEN index_10 = "24" THEN 125 | (0 << 8) | (125 << 16)
        WHEN index_10 = "36" THEN 150 | (0 << 8) | (100 << 16)
        WHEN index_10 = "25" THEN 175 | (0 << 8) | (75 << 16)
        WHEN index_10 = "37" THEN 150 | (0 << 8) | (0 << 16)
        WHEN index_10 = "30" THEN 150 | (50 << 8) | (0 << 16)
        WHEN index_10 = "26" THEN 175 | (75 << 8) | (0 << 16)
        WHEN index_10 = "31" THEN 150 | (100 << 8) | (0 << 16)
        WHEN index_10 = "19" THEN 255 | (255 << 8) | (125 << 16)
        WHEN index_10 = "40" THEN 125 | (255 << 8) | (175 << 16)
        WHEN index_10 = "41" THEN 125 | (125 << 8) | (255 << 16)
        ELSE -CAST(index_10 AS INTEGER)
    END primary_color,
    CASE
        WHEN index_11 = "0" THEN 125 | (255 << 8) | (0 << 16)
        WHEN index_11 = "1" THEN 0 | (255 << 8) | (0 << 16)
        WHEN index_11 = "2" THEN 0 | (255 << 8) | (125 << 16)
        WHEN index_11 = "3" THEN 0 | (255 << 8) | (255 << 16)
        WHEN index_11 = "16" THEN 0 | (200 << 8) | (255 << 16)
        WHEN index_11 = "4" THEN 0 | (125 << 8) | (255 << 16)
        WHEN index_11 = "5" THEN 0 | (0 << 8) | (255 << 16)
        WHEN index_11 = "6" THEN 125 | (0 << 8) | (255 << 16)
        WHEN index_11 = "13" THEN 185 | (0 << 8) | (255 << 16)
        WHEN index_11 = "7" THEN 255 | (0 << 8) | (255 << 16)
        WHEN index_11 = "8" THEN 255 | (0 << 8) | (125 << 16)
        WHEN index_11 = "9" THEN 255 | (0 << 8) | (0 << 16)
        WHEN index_11 = "29" THEN 255 | (75 << 8) | (0 << 16)
        WHEN index_11 = "10" THEN 255 | (125 << 8) | (0 << 16)
        WHEN index_11 = "14" THEN 255 | (185 << 8) | (0 << 16)
        WHEN index_11 = "11" THEN 255 | (255 << 8) | (0 << 16)
        WHEN index_11 = "12" THEN 255 | (255 << 8) | (255 << 16)
        WHEN index_11 = "17" THEN 175 | (175 << 8) | (175 << 16)
        WHEN index_11 = "18" THEN 80 | (80 << 8) | (80 << 16)
        WHEN index_11 = "15" THEN 0 | (0 << 8) | (0 << 16)
        WHEN index_11 = "27" THEN 125 | (125 << 8) | (0 << 16)
        WHEN index_11 = "32" THEN 100 | (150 << 8) | (0 << 16)
        WHEN index_11 = "28" THEN 75 | (175 << 8) | (0 << 16)
        WHEN index_11 = "38" THEN 0 | (150 << 8) | (0 << 16)
        WHEN index_11 = "20" THEN 0 | (175 << 8) | (75 << 16)
        WHEN index_11 = "33" THEN 0 | (150 << 8) | (100 << 16)
        WHEN index_11 = "21" THEN 0 | (125 << 8) | (125 << 16)
        WHEN index_11 = "34" THEN 0 | (100 << 8) | (150 << 16)
        WHEN index_11 = "22" THEN 0 | (75 << 8) | (175 << 16)
        WHEN index_11 = "39" THEN 0 | (0 << 8) | (150 << 16)
        WHEN index_11 = "23" THEN 75 | (0 << 8) | (175 << 16)
        WHEN index_11 = "35" THEN 100 | (0 << 8) | (150 << 16)
        WHEN index_11 = "24" THEN 125 | (0 << 8) | (125 << 16)
        WHEN index_11 = "36" THEN 150 | (0 << 8) | (100 << 16)
        WHEN index_11 = "25" THEN 175 | (0 << 8) | (75 << 16)
        WHEN index_11 = "37" THEN 150 | (0 << 8) | (0 << 16)
        WHEN index_11 = "30" THEN 150 | (50 << 8) | (0 << 16)
        WHEN index_11 = "26" THEN 175 | (75 << 8) | (0 << 16)
        WHEN index_11 = "31" THEN 150 | (100 << 8) | (0 << 16)
        WHEN index_11 = "19" THEN 255 | (255 << 8) | (125 << 16)
        WHEN index_11 = "40" THEN 125 | (255 << 8) | (175 << 16)
        WHEN index_11 = "41" THEN 125 | (125 << 8) | (255 << 16)
        ELSE -CAST(index_11 AS INTEGER)
    END secondary_color, secret_coins, account_id, user_coins, index_18, index_19, youtube_url, cube_index, ship_index, ball_index, ufo_index, wave_index, robot_index, has_glow, index_29, global_rank, index_31, spider_index, twitter_url, twitch_url, diamonds, death_effect_index, mod_level, index_50
    FROM temp_table;

DROP TABLE temp_table;

PRAGMA foreign_keys=on;