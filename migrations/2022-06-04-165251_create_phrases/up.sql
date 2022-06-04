CREATE TABLE phrases (
    id SERIAL PRIMARY KEY,
    words TEXT[] NOT NULL,
    words_hash BIGINT NOT NULL UNIQUE
);