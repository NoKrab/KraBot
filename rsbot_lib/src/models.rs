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
}

#[derive(Insertable, Debug)]
#[table_name = "guilds"]
pub struct NewGuild {
    pub id: i64,
    pub prefix: Option<String>,
}

// #[derive(Queryable, AsChangeset, Clone, Debug)]
// #[changeset_options(treat_none_as_null = "true")]
// #[table_name = "shards"]
// pub struct Shard {
//     pub id: i64,
//     #[sql_type = "Timestamp"]
//     pub chrono_timestamp: NaiveDateTime,
// }

// #[derive(Insertable, Debug)]
// #[table_name = "shards"]
// pub struct NewShard {
//     pub id: i64,
//     #[sql_type = "Timestamp"]
//     pub chrono_timestamp: NaiveDateTime,
// }
