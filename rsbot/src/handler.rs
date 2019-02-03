use config;
use config::Config;
use database::data::get_guild_ids;
use lazy_static;
use pg_backend;
use serenity::client::bridge::gateway::ShardManager;
use serenity::client::CACHE;
use serenity::framework::standard::{help_commands, Args, CommandOptions, DispatchError, HelpBehaviour, StandardFramework};
use serenity::http;
use serenity::model::event::ResumedEvent;
use serenity::model::gateway::Ready;
use serenity::prelude::Mutex;
use serenity::prelude::*;
use sqlite;
use util::threads::uptime;
use DIESEL_PG;

lazy_static! {
    static ref SQLITE_PATH: (String, String) = Config::get_sqlite_path(config::CONFIG_PATH);
}

pub struct Handler;

impl EventHandler for Handler {
    fn ready(&self, ctx: Context, ready: Ready) {
        ctx.set_game("Some text");
        if let Some(shard) = ready.shard {
            // Note that array index 0 is 0-indexed, while index 1 is 1-indexed.
            //
            // This may seem unintuitive, but it models Discord's behaviour.
            info!("{} is connected on shard {}/{}!", ready.user.name, shard[0], shard[1],);
            let con = sqlite::create_connection(&*SQLITE_PATH);
            sqlite::create_bot_table(&con);
            sqlite::insert_timestamp(&con, shard[0] as i64, ready.user.name);
            let _ = con.close().expect("Failed to close connection");
            if let Err(e) = DIESEL_PG.new_shard(shard[0] as i32) {
                error!("{}", e);
            }
            // this is actually a terrible idea
            // if !Path::new("./log").exists() {
            //     fs::create_dir("./log").expect("Error creating folder")
            // };
            // let mut file = File::create("./log/startuptime.log").expect("Error creating file!");
            // file.write_fmt(format_args!("{:?}", Utc::now()))
            //     .expect("Error writing to file!");
        }
        // Since the bot started, the CACHE should be filled with information kappa
        let guilds = CACHE.read().guilds.len();
        debug!("Guilds in the Cache: {}", guilds);
        let guild_ids = get_guild_ids();
        for guild in guild_ids {
            if let Err(e) = DIESEL_PG.new_guild(guild) {
                error!("Failed creating new guild {}", e);
            };
        }
        pg_backend::init_db();
        uptime::init(ctx);
    }
    fn resume(&self, _: Context, _: ResumedEvent) {
        info!("Resumed");
    }
}
