table! {
    instruments (symbol) {
        symbol -> Varchar,
        fetched -> Timestamptz,
    }
}

table! {
    quotes (symbol, date) {
        symbol -> Varchar,
        date -> Timestamptz,
        open -> Float8,
        high -> Float8,
        low -> Float8,
        close -> Float8,
        volume -> Int4,
    }
}

allow_tables_to_appear_in_same_query!(
    instruments,
    quotes,
);
