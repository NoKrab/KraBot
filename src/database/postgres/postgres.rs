use postgres::stmt::Statement;
use postgres::types::ToSql;
use postgres::{Connection, Error, TlsMode};
use postgres::rows::Rows;
use r2d2;
use r2d2_postgres::{PostgresConnectionManager, TlsMode as r2d2_TlsMode};
use serenity::client::CACHE;
use serenity::model::id::GuildId;
use serenity::model::prelude::*;
use serenity::prelude::*;
use uuid::Uuid;
use CONFIG;

use super::tables::*;
use database::data;

lazy_static! {
    pub static ref PGPOOL: r2d2::Pool<PostgresConnectionManager> = create_postgres_pool();
}

fn create_postgres_pool() -> r2d2::Pool<PostgresConnectionManager> {
    let manager = PostgresConnectionManager::new(&*CONFIG.required.pg_connection_url, r2d2_TlsMode::None).unwrap();
    let pool = r2d2::Pool::new(manager).unwrap();
    pool
}

fn create_tables(pool: &r2d2::Pool<PostgresConnectionManager>) {
    let conn = pool.get().unwrap();
    conn.batch_execute(
        "CREATE TABLE IF NOT EXISTS guild
            (
                guild_id   BIGINT NOT NULL,
                PRIMARY KEY (guild_id)
            );

            CREATE TABLE IF NOT EXISTS bot
            (
                shard_id       INTEGER NOT NULL,
                startup_time   TIMESTAMP,
                PRIMARY KEY (shard_id)
            );

            CREATE TABLE IF NOT EXISTS settings
            (
                guild_id            BIGINT NOT NULL,
                yt_search_results   INTEGER DEFAULT 5 NOT NULL,
                imgur_album_id      VARCHAR,
                PRIMARY KEY (guild_id),
                FOREIGN KEY (guild_id) REFERENCES guild (guild_id)
            );

            CREATE INDEX IF NOT EXISTS settings_FKIndex1
                ON settings (guild_id);

            CREATE INDEX IF NOT EXISTS IFK_Rel_01
                ON settings (guild_id);",
    ).unwrap();
}

fn insert_guild_ids(pool: &r2d2::Pool<PostgresConnectionManager>, ids: Vec<u64>) {
    let conn = pool.get().unwrap();
    // DOES NOT WORK
    //    let stmt = conn.prepare("INSERT INTO guild (guild_id) VALUES (guild_id = $1) ON CONFLICT DO NOTHING").unwrap();
    //    // no ToSql for u64 so i64 will be used
    //    for g in ids {
    //        let retarded = g as i64;
    //        stmt.execute(&[&retarded]).unwrap();
    //    }
    for g in ids {
        let g = g as i64;
        conn.execute("INSERT INTO guild (guild_id) VALUES ($1) ON CONFLICT DO NOTHING", &[&g]).unwrap();
        conn.execute("INSERT INTO settings (guild_id) VALUES ($1) ON CONFLICT DO NOTHING", &[&g]).unwrap();
    }
}

pub fn init_db() {
    let pool = &PGPOOL.clone();
    let guild_ids = data::get_guild_ids();
    debug!("Guild_ids: {:?}", guild_ids);
    create_tables(pool);
    insert_guild_ids(pool, guild_ids);
}

pub fn execute_sql(sql: &str, params: &[&ToSql]) {
    let pool = &PGPOOL.clone();
    let conn = pool.get().unwrap();
    conn.execute(sql, params).unwrap();
}

pub fn query_sql(sql: &str, params: &[&ToSql]) -> Result<Rows, Box<Error>> {
    let pool = &PGPOOL.clone();
    let conn = pool.get().unwrap();
    let stmt = conn.prepare(sql).unwrap();
    let rows = stmt.query(params)?;
    Ok(rows)
}