PRAGMA foreign_keys=off;

BEGIN TRANSACTION;

ALTER TABLE partial_level RENAME TO temp_table;
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
INSERT INTO partial_level (level_id,
                           level_name,
                           description,
                           level_version,
                           creator_id,
                           difficulty,
                           downloads,
                           main_song,
                           gd_version,
                           likes,
                           level_length,
                           stars,
                           featured,
                           copy_of,
                           custom_song_id,
                           coin_amount,
                           coins_verified,
                           stars_requested,
                           is_epic,
                           index_43,
                           object_amount,
                           index_46,
                           index_47)
  SELECT level_id,
         level_name,
         description,
         level_version,
         creator_id,
         difficulty,
         downloads,
         main_song,
         gd_version,
         likes,
         level_length,
         stars,
         featured,
         copy_of,
         custom_song_id,
         coin_amount,
         coins_verified,
         stars_requested,
         is_epic,
         index_43,
         object_amount,
         index_46,
         index_47
  FROM temp_table;
DROP TABLE temp_table;
COMMIT;


PRAGMA foreign_keys=on;