DROP TABLE IF EXISTS main_package;

CREATE TABLE main_package (
    name TEXT NOT NULL PRIMARY KEY
) WITHOUT ROWID;

CREATE TABLE static_files (
    name TEXT NOT NULL PRIMARY KEY,
    content_type TEXT NOT NULL,
    content_hash TEXT NULL,
) WITHOUT ROWID;
