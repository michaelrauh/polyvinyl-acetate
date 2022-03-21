CREATE TABLE pairs (
    id SERIAL PRIMARY KEY,
    first_word TEXT NOT NULL,
    second_word TEXT NOT NULL,
    first_word_hash BIGINT NOT NULL,
    second_word_hash BIGINT NOT NULL,
    UNIQUE(first_word_hash, second_word_hash)
);