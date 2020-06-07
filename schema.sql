CREATE TABLE IF NOT EXISTS users (
    username    TEXT NOT NULL UNIQUE PRIMARY KEY,
    nickname    TEXT NOT NULL,
    password    TEXT NOT NULL,
    is_admin    BOOLEAN NOT NULL DEFAULT 'f'
);