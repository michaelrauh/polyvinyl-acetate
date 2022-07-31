CREATE TABLE pairs (
    id SERIAL PRIMARY KEY,
    first_word INTEGER NOT NULL,
    second_word INTEGER NOT NULL,
    pair_hash BIGINT NOT NULL UNIQUE
);