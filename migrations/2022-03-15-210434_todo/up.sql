CREATE TABLE todos (
    id SERIAL PRIMARY KEY,
    domain VARCHAR(64) NOT NULL,
    other INTEGER NOT NULL
);