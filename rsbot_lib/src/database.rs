use self::r2d2::{ConnectionManager, Pool, PooledConnection};
use chrono::{DateTime, NaiveDateTime, TimeZone, Utc};
use diesel::{
    pg::{upsert::*, PgConnection},
    prelude::*,
    r2d2,
    result::Error,
};
use dotenv::dotenv;
use models::*;
use std::env;

#[derive(Clone)]
pub struct ConnectionPool {
    pool: Pool<ConnectionManager<PgConnection>>,
}

embed_migrations!("./migrations");

impl Default for ConnectionPool {
    fn default() -> Self {
        ConnectionPool::new()
    }
}

impl ConnectionPool {
    pub fn new() -> ConnectionPool {
        let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set in the enviroment.");
        let manager = ConnectionManager::<PgConnection>::new(database_url);

        let pool = Pool::builder().build(manager).expect("Failed to create pool.");
        info!("Running pending migrations...");
        let conn = (&pool).get().unwrap();
        if let Err(e) = embedded_migrations::run_with_output(&conn, &mut std::io::stdout()) {
            eprintln!("[DB:embedded_migrations] Error while running pending migrations: {}", e);
        };

        ConnectionPool { pool }
    }

    // return connection
    pub fn connection(&self) -> PooledConnection<ConnectionManager<PgConnection>> {
        self.pool.get().unwrap()
    }

    pub fn ping(&self) -> bool {
        let conn = self.connection();
        return diesel::sql_query(r#"SELECT 1"#).execute(&conn).is_ok();
    }

    pub fn new_guild(&self, guild_id: i64) -> Result<Guild, Error> {
        use schema::guilds;
        let new_guild_obj = NewGuild {
            id: guild_id as i64,
            prefix: None,
            youtube_results: None,
            imgur_album_id: None,
        };
        let conn = self.connection();
        diesel::insert_into(guilds::table).values(&new_guild_obj).get_result::<Guild>(&conn)
    }

    pub fn update_guild(&self, guild: &Guild) {
        use schema::{guilds, guilds::dsl::*};
        let conn = self.connection();
        if let Err(e) = diesel::update(guilds::table).filter(id.eq(guild.id)).set(guild).execute(&conn) {
            error!("Error updating guild {}", e)
        }
    }

    // one could use diesel::dsl::now to insert database timestamp,
    // but we care about the timestamp on our side
    pub fn new_shard(&self, shard_id: i32) -> Result<usize, Error> {
        use schema::shards::dsl::*;
        let new_shard_obj = Shard {
            id: shard_id as i32,
            chrono_timestamp: Utc::now().naive_utc(),
        };
        let conn = self.connection();

        diesel::insert_into(shards)
            .values(&new_shard_obj)
            .on_conflict(id)
            .do_update()
            .set(chrono_timestamp.eq(Utc::now().naive_utc()))
            .execute(&conn)
    }
    pub fn get_shard_timestamp(&self, shard_id: i32) -> Result<Shard, Error> {
        use schema::shards::dsl::*;
        let conn = self.connection();
        let shard = shards.filter(id.eq(shard_id as i32)).first::<Shard>(&conn);
        if let Err(e) = shard {
            match e {
                _ => {
                    error!("Failed retrieving the timestamp from the database: {}", e);
                    return Err(e);
                }
            }
        } else {
            return shard;
        }
    }
}

pub fn establish_connection() -> PgConnection {
    dotenv().ok();

    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    PgConnection::establish(&database_url).expect(&format!("Error connecting to {}", database_url))
}
