-- Your SQL goes here
CREATE TABLE quotes (
    symbol VARCHAR NOT NULL,
    date TIMESTAMPTZ NOT NULL,
    open FLOAT NOT NULL,
    high FLOAT NOT NULL,
    low FLOAT NOT NULL,
    close FLOAT NOT NULL,
    volume INTEGER NOT NULL,
    PRIMARY KEY (symbol, date)
);