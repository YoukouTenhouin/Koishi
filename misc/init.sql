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
       timestamp INTEGER NOT NULL,
       FOREIGN KEY (room) REFERENCES room(id)
);
