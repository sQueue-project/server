CREATE TABLE pretracks (
    id INT NOT NULL PRIMARY KEY AUTO_INCREMENT,
    uuid BLOB NOT NULL,
    name TEXT NOT NULL,
    artist TEXT NOT NULL,
    duration BIGINT NOT NULL,
    thumbnail_url TEXT NOT NULL,
    platform TEXT NOT NULL,
    platform_track_id TEXT NOT NULL
);