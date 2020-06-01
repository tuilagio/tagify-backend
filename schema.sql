CREATE TABLE IF NOT EXISTS todos (
    id          SERIAL PRIMARY KEY,
    description TEXT NOT NULL,
    date        TEXT NOT NULL,
    progress    INT NOT NULL
);
