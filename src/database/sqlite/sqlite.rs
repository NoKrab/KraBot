use time;
use time::Timespec;
use std::fs;
use std::path::Path;
use rusqlite::Connection;
use rusqlite::types::FromSql;

#[derive(Debug)]
struct Bot {
    id: i64,
    name: String,
    time_created: Timespec,
    chrono_timestamp: String,
}

pub fn create_connection(&(ref path, ref loc): &(String, String)) -> Connection {
    if !Path::new(&loc).exists() {
        fs::create_dir_all(loc).expect("Error creating directory");
    }
    let con = Connection::open(path).unwrap();
    con
}

pub fn select_shard_uptime(con: Connection, shard: i64) -> Result<Vec<String>, ()> {
    let mut stmt = con.prepare("SELECT chrono_timestamp FROM bot WHERE id = :id").unwrap();
    let mut rows = stmt.query_named(&[(":id", &shard)]).unwrap();
    let mut stamp = Vec::new();
    while let Some(row) = rows.next() {
        let row = row.unwrap();
        stamp.push(row.get(0));
    }
    Ok(stamp)
}

pub fn create_bot_table(con: Connection) -> Connection {
    con.execute(
        "CREATE TABLE IF NOT EXISTS bot (id INTEGER PRIMARY KEY, name TEXT NOT NULL, time_created TEXT NOT NULL, chrono_timestamp TEXT NOT NULL)",
        &[],
    ).unwrap();
    con
}

pub fn insert_timestamp(con: Connection, id: u64, name: String, timestamp: String) -> Connection {
    let t = time::get_time();
    con.execute(
        "INSERT OR REPLACE INTO bot(id, name, time_created, chrono_timestamp) VALUES ($1, $2, $3, $4)",
        &[&id.to_string(), &name, &t, &timestamp],
    ).unwrap();
    con
}

pub fn get_timestamp(con: Connection) {
    let mut stmt = con.prepare("SELECT id, name, time_created, chrono_timestamp FROM bot")
        .unwrap();
    let bot_iter = stmt.query_map(&[], |row| Bot {
        id: row.get(0),
        name: row.get(1),
        time_created: row.get(2),
        chrono_timestamp: row.get(3),
    }).unwrap();
    for bot in bot_iter {
        println!("Found bot {:?}", bot.unwrap());
    }
}
