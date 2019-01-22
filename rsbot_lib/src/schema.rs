table! {
    guilds (id) {
        id -> Int8,
        prefix -> Nullable<Text>,
        youtube_results -> Nullable<Int4>,
        imgur_album_id -> Nullable<Varchar>,
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
