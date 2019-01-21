table! {
    guilds (id) {
        id -> Int8,
        prefix -> Nullable<Text>,
    }
}

table! {
    shards (id) {
        id -> Int4,
        chrono_timestamp -> Timestamp,
    }
}

allow_tables_to_appear_in_same_query!(
    guilds,
    shards,
);
