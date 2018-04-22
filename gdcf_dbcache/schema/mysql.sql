CREATE TABLE newgrounds_song (
    song_id BIGINT PRIMARY KEY NOT NULL,
    song_name TEXT NOT NULL,
    artist TEXT NOT NULL,
    filesize DOUBLE NOT NULL,
    alt_artist TEXT NULL DEFAULT NULL,
    banned BOOL NOT NULL DEFAULT false,
    download_link TEXT NOT NULL,
    internal_id BIGINT UNIQUE NOT NULL
);