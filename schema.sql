CREATE TABLE IF NOT EXISTS users (
    id SERIAL PRIMARY KEY,
    username    TEXT UNIQUE NOT NULL,
    nickname    TEXT NOT NULL,
    password    TEXT NOT NULL,
    role    TEXT NOT NULL DEFAULT 'tagger'
);

INSERT INTO users (username, nickname, password, role)
VALUES ('admin', 'realAdmin', '$argon2i$v=19$m=4096,t=3,p=1$AJFNXjA2e/WtFQ6LCm/tPIJ/M9HAjtlCgFmUIxHizoA$K/xiVpnYgMKC5mEu5B8msuq50spwHiBcl/MYTN4glrw', 'admin');

INSERT INTO users (username, nickname, password, role)
VALUES ('user1', 'nickuser1', '$argon2i$v=19$m=4096,t=3,p=1$AJFNXjA2e/WtFQ6LCm/tPIJ/M9HAjtlCgFmUIxHizoA$K/xiVpnYgMKC5mEu5B8msuq50spwHiBcl/MYTN4glrw', 'tagger');