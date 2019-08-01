-- sqlite doesnt have an alter table drop column thing >.>

PRAGMA foreign_keys=off;

BEGIN TRANSACTION;
ALTER TABLE song_meta RENAME TO temp_table;
CREATE TABLE song_meta (
    song_id INTEGER PRIMARY KEY,
    cached_at INTEGER
);
INSERT INTO song_meta (song_id, cached_at)
  SELECT song_id, cached_at
  FROM temp_table;
DROP TABLE temp_table;
COMMIT;


BEGIN TRANSACTION;
ALTER TABLE creator_meta RENAME TO temp_table;
CREATE TABLE creator_meta (
    user_id INTEGER PRIMARY KEY,
    cached_at INTEGER
);
INSERT INTO creator_meta (user_id, cached_at)
  SELECT user_id, cached_at
  FROM temp_table;
DROP TABLE temp_table;
COMMIT;


BEGIN TRANSACTION;
ALTER TABLE creator_meta RENAME TO temp_table;
CREATE TABLE creator_meta (
    user_id INTEGER PRIMARY KEY,
    cached_at INTEGER
);
INSERT INTO creator_meta (user_id, cached_at)
  SELECT user_id, cached_at
  FROM temp_table;
DROP TABLE temp_table;
COMMIT;

BEGIN TRANSACTION;
ALTER TABLE partial_level_meta RENAME TO temp_table;
CREATE TABLE partial_level_meta (
    level_id INTEGER PRIMARY KEY,
    cached_at INTEGER
);
INSERT INTO partial_level_meta (level_id, cached_at)
  SELECT  level_id, cached_at
  FROM temp_table;
DROP TABLE temp_table;
COMMIT;


BEGIN TRANSACTION;
ALTER TABLE level_list_meta RENAME TO temp_table;
CREATE TABLE level_list_meta (
    request_hash INTEGER PRIMARY KEY,
    cached_at INTEGER
);
INSERT INTO level_list_meta (request_hash, cached_at)
    SELECT request_hash, cached_at
    FROM temp_table;
DROP TABLE temp_table;
COMMIT;


BEGIN TRANSACTION;
ALTER TABLE level_meta RENAME TO temp_table;
CREATE TABLE level_meta (
    level_id INTEGER PRIMARY KEY,
    cached_at INTEGER
);
INSERT INTO level_meta (level_id, cached_at)
  SELECT  level_id, cached_at
  FROM temp_table;
DROP TABLE temp_table;
COMMIT;


BEGIN TRANSACTION;
ALTER TABLE profile_meta RENAME TO temp_table;
CREATE TABLE profile_meta (
    user_id INTEGER PRIMARY KEY,
    cached_at INTEGER
);
INSERT INTO profile_meta (user_id, cached_at)
  SELECT  user_id, cached_at
  FROM temp_table;
DROP TABLE temp_table;
COMMIT;


PRAGMA foreign_keys=on;