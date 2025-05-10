DROP TABLE IF EXISTS video;
DROP TABLE IF EXISTS room;

CREATE TABLE IF NOT EXISTS room (
       id INTEGER PRIMARY KEY NOT NULL,
       short_id INTEGER,
       username TEXT NOT NULL,
       image TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS video (
       uuid BLOB PRIMARY KEY NOT NULL,
       title TEXT NOT NULL,
       cover TEXT,
       room INTEGER NOT NULL,
       stream_time INTEGER NOT NULL,
       record_time INTEGER NOT NULL,
       len INTEGER,
       restricted INTEGER NOT NULL DEFAULT 0,
       restricted_hash TEXT,
       FOREIGN KEY (room) REFERENCES room(id)
);
