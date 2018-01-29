use time;
use time::Timespec;
use std::fs;
use std::path::Path;
use rusqlite::Connection;

#[derive(Debug)]
struct Person {
    id: i32,
    name: String,
    time_created: Timespec,
    data: Option<Vec<u8>>,
}

pub fn create_connection(&(ref path, ref loc): &(String, String)) -> Connection {
    if !Path::new(&loc).exists() {
        fs::create_dir_all(loc).expect("Error creating directory");
    }
    let con = Connection::open(path).unwrap();
    con
}

pub fn create_bot_table(con: Connection) -> Connection {
    con.execute(
        "CREATE TABLE IF NOT EXISTS bot (
                id                  INTEGER PRIMARY KEY,
                name                TEXT NOT NULL,
                time_created        TEXT NOT NULL,
                chrono_timestamp    TEXT NOT NULL
                )",
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

//pub fn get_timestamp(con: Connection) -> (Connection, String[], String[]) {
//let mut stmt
//let timestamp = [String::from("Kappa"), String: from("Keppo")];
//let id = [0, 1];
//(con, id[], timestamp[])
//}

pub fn execute_statement() {
    let conn = Connection::open("./db/rsbot.db").unwrap();
    conn.execute(
        "CREATE TABLE IF NOT EXISTS person (id INTEGER PRIMARY KEY, name TEXT NOT NULL, time_created TEXT NOT NULL, data BLOB)",
        &[],
    ).unwrap();
    let me = Person {
        id: 0,
        name: "Steven".to_string(),
        time_created: time::get_time(),
        data: None,
    };
    conn.execute(
        "INSERT INTO person (name, time_created, data)
                  VALUES ($1, $2, $3)",
        &[&me.name, &me.time_created, &me.data],
    ).unwrap();

    let mut stmt = conn.prepare("SELECT id, name, time_created, data FROM person")
        .unwrap();
    let mut person_iter = stmt.query_map(&[], |row| Person {
        id: row.get(0),
        name: row.get(1),
        time_created: row.get(2),
        data: row.get(3),
    }).unwrap();

    for person in person_iter {
        println!("Found person {:?}", person.unwrap());
    }
    //    it's actually broken
    //    conn.close();
}
