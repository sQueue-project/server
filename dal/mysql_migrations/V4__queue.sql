CREATE TABLE queue (
    id INT PRIMARY KEY NOT NULL AUTO_INCREMENT,
    track_uuid BLOB NOT NULL,
    room_uuid BLOB NOT NULL,
    idx INT NOT NULL,
    added_by BLOB NOT NULL
);