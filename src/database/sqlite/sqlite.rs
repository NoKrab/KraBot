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

pub fn create_connection((path, loc): (String, String)) -> Connection {
    if !Path::new(&loc).exists() {
        fs::create_dir_all(loc).expect("Error creating directory");
    }
    let con = Connection::open(path).unwrap();
    con
}

pub fn test(con: Connection) -> Connection {
    con.execute(
        "CREATE TABLE IF NOT EXISTS bot (
                id          INTEGER PRIMARY KEY,
                name        TEXT NOT NULL,
                timestamp   TEXT NOT NULL
                )",
        &[],
    ).unwrap();
    con
}

pub fn test2(con: Connection, id: u64, name: String, timestamp: String) -> Connection {
    con.execute("INSERT OR IGNORE INTO bot(id, name, timestamp)
                        VALUES ($1, $2, $3)",
                        &[&id.to_string(), &name, &timestamp],
    ).unwrap();
    // UPDATE my_table SET age = 34 WHERE name='Karen'
    // con.execute("UPDATE bot SET timestamp=$2 WHERE id=$1
    //                 VALUES ($1, $2)",
    //             &[&id.to_string(), &timestamp],
    // ).unwrap();
    con
} 

pub fn execute_statement() {
    let conn = Connection::open("./db/rsbot.db").unwrap();
    conn.execute(
        "CREATE TABLE IF NOT EXISTS person (
                  id              INTEGER PRIMARY KEY,
                  name            TEXT NOT NULL,
                  time_created    TEXT NOT NULL,
                  data            BLOB
                  )",
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
