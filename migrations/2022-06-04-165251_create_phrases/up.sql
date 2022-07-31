CREATE TABLE phrases (
    id SERIAL PRIMARY KEY,
    words INTEGER[] NOT NULL,
    words_hash BIGINT NOT NULL UNIQUE
);