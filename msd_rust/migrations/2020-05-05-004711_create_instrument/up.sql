-- Your SQL goes here
CREATE TABLE instruments (
    symbol VARCHAR PRIMARY KEY NOT NULL,
    fetched TIMESTAMPTZ NOT NULL
);