use chrono::prelude::*;
use chrono::{DateTime, Duration};
use std::fs;
use std::path::Path;
use rusqlite::{Connection, Error};
use rusqlite::types::ToSql;

//#[derive(Debug)]
//struct Bot {
//    id: i64,
//    name: String,
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

pub fn select_shard_uptime(
    con: &Connection,
    shard: i64,
) -> Result<Duration, Error> {
    let mut stmt = con.prepare("SELECT chrono_timestamp FROM bot WHERE id = :id")?;
    let mut rows = stmt.query_named(&[(":id", &shard)])?;
    let mut stamp: Vec<String> = Vec::new();
    while let Some(row) = rows.next() {
        let row = row?;
        stamp.push(row.get(0));
    }
    if !stamp.is_empty() {
        let duration = Utc::now().signed_duration_since(stamp[0].parse::<DateTime<Utc>>().expect("Failed parsing timestamp"));
        Ok(duration)
    } else {
        error!("Could not retrieve timestamp");
        Ok(Duration::seconds(0))
    }
}

pub fn create_bot_table(con: &Connection) {
    match con.execute("CREATE TABLE IF NOT EXISTS bot (id INTEGER PRIMARY KEY UNIQUE, name TEXT NOT NULL, chrono_timestamp TEXT NOT NULL)", &[]) {
        Ok(_) => (),
        Err(why) => error!("Error creating bot table: {}", why)
    }
}

pub fn insert_timestamp(con: &Connection, id: i64, name: String) {
    let utc = Utc::now();
    match con.execute("INSERT OR REPLACE INTO bot(id, name, chrono_timestamp) VALUES ($1, $2, $3)", &[&id.to_sql().unwrap(), &name, &utc.to_sql().unwrap()]) {
        Ok(i) => info!("Inserted {} row(s)", i),
        Err(why) => error!("Error: {}", why)
    }
}

//pub fn get_timestamp(con: Connection) {
//    let mut stmt = con.prepare("SELECT id, name, chrono_timestamp FROM bot")
//        .unwrap();
//    let bot_iter = stmt.query_map(&[], |row| Bot {
//        id: row.get(0),
//        name: row.get(1),
//        chrono_timestamp: row.get(2),
//    }).unwrap();
//    for bot in bot_iter {
//        println!("Found bot {:?}", bot.unwrap());
//    }
//}
