CREATE TABLE newgrounds_song IF NOT EXISTS (
    song_id INTEGER PRIMARY KEY NOT NULL,
    song_name TEXT NOT NULL,
    artist TEXT NOT NULL,
    filesize REAL NOT NULL,
    alt_artist TEXT DEFAULT NULL,
    banned INTEGER NOT NULL DEFAULT 0,
    download_link TEXT NOT NULL,
    internal_id INTEGER UNIQUE NOT NULL
);