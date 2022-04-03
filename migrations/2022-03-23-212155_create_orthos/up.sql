CREATE TABLE orthotopes (
    id SERIAL PRIMARY KEY,
    information BYTEA NOT NULL,
    origin TEXT NOT NULL,
    hop TEXT[] NOT NULL,
    contents TEXT[] NOT NULL,
    info_hash BIGINT NOT NULL UNIQUE
);