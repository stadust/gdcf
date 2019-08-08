DROP TABLE profile_meta;

CREATE TABLE profile_meta (
    user_id INTEGER PRIMARY KEY,
    cached_at TIMESTAMP WITHOUT TIME ZONE,
    absent BOOL NOT NULL DEFAULT FALSE
);