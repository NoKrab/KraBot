use self::r2d2::{ConnectionManager, Pool, PooledConnection};
use diesel::{pg::PgConnection, prelude::*, r2d2, result::Error};
use dotenv::dotenv;
use std::env;

use models::*;

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
        let new_guild_obj = NewGuild { id: guild_id as i64, prefix: None };
        let conn = self.connection();
        diesel::insert_into(guilds::table).values(&new_guild_obj).get_result::<Guild>(&conn)
    }
}

pub fn establish_connection() -> PgConnection {
    dotenv().ok();

    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    PgConnection::establish(&database_url).expect(&format!("Error connecting to {}", database_url))
}
