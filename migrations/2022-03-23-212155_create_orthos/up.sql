CREATE TABLE orthotopes (
    id SERIAL PRIMARY KEY,
    information BYTEA NOT NULL,
    origin INTEGER NOT NULL,
    hop INTEGER[] NOT NULL,
    contents INTEGER[] NOT NULL,
    info_hash BIGINT NOT NULL UNIQUE
);