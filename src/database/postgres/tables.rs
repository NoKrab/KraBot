use chrono::prelude::*;
use uuid::Uuid;

#[derive(Debug)]
pub struct Person {
    pub uuid: Uuid,
    pub name: String,
    pub data: Option<Vec<u8>>,
}

#[derive(Debug)]
pub struct Guild {
    pub guild_id: u64,
}

#[derive(Debug)]
pub struct Settings {
    pub guild_id: u64,
    pub yt_search_results: u64,
}

#[derive(Debug)]
pub struct Bot {
    shard_id: u64,
    startup_time: DateTime<Utc>,
}
