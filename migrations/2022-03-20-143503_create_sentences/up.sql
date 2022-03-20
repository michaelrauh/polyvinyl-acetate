CREATE TABLE sentences (
    id SERIAL PRIMARY KEY,
    sentence TEXT NOT NULL,
    sentence_hash BIGINT UNIQUE NOT NULL
);