
DROP TABLE IF EXISTS users;
/* DROP TYPE IF EXISTS ROLE; */

/* CREATE TYPE ROLE AS ENUM ('admin', 'tagger'); */
CREATE TABLE IF NOT EXISTS users (
    id SERIAL PRIMARY KEY,
    username    TEXT UNIQUE NOT NULL,
    nickname    TEXT NOT NULL,
    password    TEXT NOT NULL,
    role    TEXT NOT NULL DEFAULT 'tagger'
);

ALTER TABLE users
  ADD CONSTRAINT namechk CHECK (char_length(username) <= 64 AND char_length(username) >= 4);


ALTER TABLE users
  ADD CONSTRAINT userchk CHECK (char_length(nickname) <= 64 AND char_length(nickname) >= 4);


