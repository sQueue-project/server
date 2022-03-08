CREATE TABLE tracks (
    id INT NOT NULL PRIMARY KEY AUTO_INCREMENT,
    uuid BLOB NOT NULL,
    room_uuid BLOB NOT NULL,
    name TEXT NOT NULL,
    artist TEXT NOT NULL,
    duration BIGINT NOT NULL,
    thumbnail_url TEXT NOT NULL,
    platform TEXT NOT NULL,
    platform_video_id TEXT NOT NULL
);