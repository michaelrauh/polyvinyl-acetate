CREATE TABLE phrases (
    id SERIAL PRIMARY KEY,
    words INTEGER[] NOT NULL,
    phrase_head BIGINT NOT NULL,
    phrase_tail BIGINT NOT NULL,
    words_hash BIGINT NOT NULL UNIQUE
);