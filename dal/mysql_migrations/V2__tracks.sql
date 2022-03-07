CREATE TABLE tracks (
    id INT NOT NULL PRIMARY KEY AUTO_INCREMENT,
    track_uuid BLOB NOT NULL,
    room_uuid BLOB NOT NULL,
    youtube_id VARCHAR(16),
    track_name TEXT NOT NULL,
    artist_name TEXT NOT NULL,
    track_duration BIGINT NOT NULL,
    queue_position INT NOT NULL,
    thumbnail_url TEXT NOT NULL,
);