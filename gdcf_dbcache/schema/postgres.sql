CREATE TABLE newgrounds_song (
    song_id BIGINT PRIMARY KEY NOT NULL,
    song_name TEXT NOT NULL,
    artist TEXT NOT NULL,
    filesize DOUBLE PRECISION NOT NULL,
    alt_artist TEXT NULL DEFAULT NULL,
    banned BOOLEAN NOT NULL DEFAULT false,
    download_link TEXT NOT NULL,
    internal_id BIGINT UNIQUE NOT NULL,
    first_cached_at TIMESTAMP WITHOUT TIME ZONE NOT NULL DEFAULT (now() AT TIME ZONE 'utc'),
    last_cached_at TIMESTAMP WITHOUT TIME ZONE NOT NULL DEFAULT (now() AT TIME ZONE 'utc')
);