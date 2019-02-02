extern crate chrono;
extern crate fern;
extern crate futures;
extern crate hyper;
extern crate hyper_tls;
#[macro_use]
extern crate lazy_static;
#[macro_use]
extern crate log;
extern crate rusqlite;
#[macro_use]
extern crate serde_derive;
extern crate serde_json;
#[macro_use]
extern crate serenity;
extern crate eval;
extern crate postgres;
extern crate r2d2;
extern crate r2d2_postgres;
extern crate rand;
extern crate regex;
extern crate reqwest;
extern crate rsbot_lib;
extern crate tokio;
extern crate tokio_core;
extern crate toml;
extern crate transient_hashmap;
extern crate typemap;
extern crate uuid;

mod audio;
mod commands;
mod config;
mod database;
mod framework;
mod util;

use rsbot_lib::database::ConnectionPool;

use database::postgres::postgres as pg_backend;
use util::threads::uptime;

use commands::voice::VoiceManager;
use config::Config;
use database::sqlite::sqlite;
//use util::network;

use std::collections::HashMap;
use std::collections::HashSet;
use std::fs;
use std::sync::Arc;
// use std::fs::File;
// use std::io::Write;
use chrono::prelude::*;
use framework::{get_framework, CommandCounter};
use serenity::client::bridge::gateway::ShardManager;
use serenity::client::CACHE;
use serenity::framework::standard::{help_commands, Args, CommandOptions, DispatchError, HelpBehaviour, StandardFramework};
use serenity::http;
use serenity::model::event::ResumedEvent;
use serenity::model::gateway::Ready;
use serenity::prelude::Mutex;
use serenity::prelude::*;
use std::path::Path;
use typemap::Key;

// What actual use does this bring?
lazy_static! {
    static ref DIESEL_PG: ConnectionPool = ConnectionPool::new();
    static ref CONFIG: Config = Config::get_config(config::CONFIG_PATH);
    static ref SQLITE_PATH: (String, String) = Config::get_sqlite_path(config::CONFIG_PATH);
}

struct Handler;

struct ShardManagerContainer;

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
        pg_backend::init_db();
        uptime::init(ctx);
    }
    fn resume(&self, _: Context, _: ResumedEvent) {
        info!("Resumed");
    }
}

impl Key for ShardManagerContainer {
    type Value = Arc<Mutex<ShardManager>>;
}

fn main() {
    match setup_logger() {
        Ok(_) => (),
        Err(why) => eprintln!("Failed to init logger: {}", why), // Since the logger isn't setup yet, we use eprintln!
    }
    debug!("Configuration file: {:?}", *CONFIG);
    debug!("SQLITE PATH: {:?}", *SQLITE_PATH);
    //    debug!("Configuration file: {:?}", *CONFIG);
    //    debug!("SQLITE PATH: {:?}", *SQLITE_PATH);

    let mut client = Client::new(&*CONFIG.required.token, Handler).expect("Error creating client");

    // let manager = client.shard_manager.clone();

    {
        let mut data = client.data.lock();
        data.insert::<CommandCounter>(HashMap::default());
        data.insert::<ShardManagerContainer>(Arc::clone(&client.shard_manager));
        data.insert::<VoiceManager>(Arc::clone(&client.voice_manager));
    }

    client.with_framework(get_framework());

    /*    thread::spawn(move || loop {
        thread::sleep(time::Duration::from_secs(30));

        let lock = manager.lock();
        let shard_runners = lock.runners.lock();

        for (id, runner) in shard_runners.iter() {
            debug!(
                "Shard ID {} is {} with a latency of {:?}",
                id, runner.stage, runner.latency,
            );
        }
    });*/

    // if let Err(why) = client.start_shards(CONFIG.required.shards).map_err(|why| error!("Client ended: {:?}", why)) {
    //     error!("Client error: {:?}", why);
    // }
    if let Err(why) = client.start_autosharded() {
        error!("Failed to start {:?}", why);
    }
}

fn setup_logger() -> Result<(), fern::InitError> {
    if !Path::new("./log").exists() {
        fs::create_dir("./log").expect("Error creating folder")
    };

    let file_config = fern::Dispatch::new().level(log::LevelFilter::Error).chain(fern::log_file("./log/rust.log")?);

    let stdout_config = fern::Dispatch::new()
        .format(|out, message, record| {
            out.finish(format_args!(
                "{}[{}][{}] {}",
                chrono::Utc::now().format("[%Y-%m-%d][%H:%M:%S]"),
                record.target(),
                record.level(),
                message
            ))
        })
        .level(log::LevelFilter::Info)
        .chain(std::io::stdout());

    stdout_config.chain(file_config).apply()?;

    debug!("Debug output enabled.");
    error!("Error output enabled.");
    info!("Info output enabled.");
    Ok(())
}
