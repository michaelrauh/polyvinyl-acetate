CREATE TABLE pairs (
    id SERIAL PRIMARY KEY,
    first_word TEXT NOT NULL,
    second_word TEXT NOT NULL,
    pair_hash BIGINT NOT NULL UNIQUE
);