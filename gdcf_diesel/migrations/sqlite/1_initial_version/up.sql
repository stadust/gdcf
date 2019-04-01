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
    song_idINTEGER PRIMARY KEY,
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