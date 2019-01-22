use chrono::NaiveDateTime;
use diesel::sql_types::*;
use schema::*;
use serde_json;

#[derive(Queryable, AsChangeset, Clone, Debug)]
#[changeset_options(treat_none_as_null = "true")]
#[table_name = "guilds"]
pub struct Guild {
    pub id: i64,
    pub prefix: Option<String>,
    pub youtube_results: Option<i32>,
    pub imgur_album_id: Option<String>,
}

#[derive(Insertable, Debug)]
#[table_name = "guilds"]
pub struct NewGuild {
    pub id: i64,
    pub prefix: Option<String>,
    pub youtube_results: Option<i32>,
    pub imgur_album_id: Option<String>,
}

#[derive(Queryable, Insertable, AsChangeset, Debug)]
#[table_name = "shards"]
pub struct Shard {
    pub id: i32,
    pub chrono_timestamp: NaiveDateTime,
}
