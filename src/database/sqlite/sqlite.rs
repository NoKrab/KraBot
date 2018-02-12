use time;
use time::Timespec;
use chrono;
use chrono::prelude::*;
use chrono::{DateTime, NaiveDateTime, TimeZone};
use std::fs;
use std::path::Path;
use rusqlite;
use rusqlite::Connection;
use rusqlite::types::{FromSql, ToSql};

//#[derive(Debug)]
//struct Bot {
//    id: i64,
//    name: String,
//    time_created: Timespec,
//    chrono_timestamp: String,
//}

pub fn create_connection(&(ref path, ref loc): &(String, String)) -> Connection {
    if !Path::new(&loc).exists() {
        fs::create_dir_all(loc).expect("Error creating directory");
    }
    let con = match Connection::open(path) {
        Ok(con) => con,
        Err(why) => {
            error!("Failed to create connection {}", why);
            panic!()
        }
    };
    con
}

pub fn select_shard_uptime(con: Connection, shard: i64) -> Result<time::Duration, rusqlite::Error> {
    let mut stmt = con.prepare("SELECT chrono_timestamp FROM bot WHERE id = :id")?;
    let mut rows = stmt.query_named(&[(":id", &shard)])?;
    let mut stamp: Vec<String> = Vec::new();
    while let Some(row) = rows.next() {
        let row = row?;
        stamp.push(row.get(0));
    }
    let duration = Utc::now().signed_duration_since(stamp[0].parse::<DateTime<Utc>>().unwrap());
    Ok(duration)
}

pub fn create_bot_table(con: &Connection) {
    match con.execute("CREATE TABLE IF NOT EXISTS bot (id INTEGER PRIMARY KEY UNIQUE, name TEXT NOT NULL, time_created TEXT NOT NULL, chrono_timestamp TEXT NOT NULL)", &[]) {
        Ok(_) => (),
        Err(why) => error!("Error creating bot table: {}", why)
    }
}

pub fn insert_timestamp(con: &Connection, id: u64, name: String) {
    let t = time::get_time();
    let utc = Utc::now();
    match con.execute("INSERT OR REPLACE INTO bot(id, name, time_created, chrono_timestamp) VALUES ($1, $2, $3, $4)", &[&id.to_string(), &name, &t, &utc.to_sql().unwrap()]) {
        Ok(i) => info!("Inserted {} row(s)", i),
        Err(why) => error!("Error: {}", why)
    }
}

//pub fn get_timestamp(con: Connection) {
//    let mut stmt = con.prepare("SELECT id, name, time_created, chrono_timestamp FROM bot")
//        .unwrap();
//    let bot_iter = stmt.query_map(&[], |row| Bot {
//        id: row.get(0),
//        name: row.get(1),
//        time_created: row.get(2),
//        chrono_timestamp: row.get(3),
//    }).unwrap();
//    for bot in bot_iter {
//        println!("Found bot {:?}", bot.unwrap());
//    }
//}
