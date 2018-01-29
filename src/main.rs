extern crate chrono;
#[macro_use]
extern crate log;
extern crate rusqlite;
#[macro_use]
extern crate serde_derive;
#[macro_use]
extern crate serenity;
extern crate time;
extern crate toml;
extern crate typemap;
#[macro_use]
extern crate lazy_static;

mod config;
mod commands;
mod database;

use config::Config;
use database::sqlite::sqlite;
use commands::voice::VoiceManager;

use std::sync::Arc;
use std::collections::HashSet;
use std::thread;
// use std::fs;
// use std::fs::File;
// use std::io::Write;
// use std::path::Path;
use std::time::Duration;
use serenity::framework::StandardFramework;
use serenity::model::event::ResumedEvent;
use serenity::model::gateway::Ready;
use serenity::prelude::*;
use serenity::http;
use chrono::prelude::*;

// What actual use does this bring?
lazy_static! {
    static ref CONFIG: Config = Config::get_config(config::CONFIG_PATH);
    static ref SQLITE_PATH: (String, String) = Config::get_sqlite_path(config::CONFIG_PATH);
}

struct Handler;

impl EventHandler for Handler {
    fn ready(&self, ctx: Context, ready: Ready) {
        ctx.set_game_name("I bims ein Rust Bot");
        if let Some(shard) = ready.shard {
            // Note that array index 0 is 0-indexed, while index 1 is 1-indexed.
            //
            // This may seem unintuitive, but it models Discord's behaviour.
            println!(
                "{} is connected on shard {}/{}!",
                ready.user.name, shard[0], shard[1],
            );
            let con = sqlite::create_connection(&*SQLITE_PATH);
            let con = sqlite::create_bot_table(con);
            let con = sqlite::insert_timestamp(
                con,
                shard[0],
                ready.user.name,
                Utc::now().to_owned().to_string(),
            );
            let _ = con.close().expect("Failed to close connection");
            // this is actually a terrible idea
            // if !Path::new("./log").exists() {
            //     fs::create_dir("./log").expect("Error creating folder")
            // };
            // let mut file = File::create("./log/startuptime.log").expect("Error creating file!");
            // file.write_fmt(format_args!("{:?}", Utc::now()))
            //     .expect("Error writing to file!");
        }
    }
    fn resume(&self, _: Context, _: ResumedEvent) {
        info!("Resumed");
    }
}


fn main() {
//    let config = Config::get_config(config::CONFIG_PATH);
//    println!("{:?}", config);
    println!("Configuration file: {:?}", *CONFIG);
    println!("SQLITE PATH: {:?}", *SQLITE_PATH);
    static SHARDS: u64 = 2;

    let mut client = Client::new(&CONFIG.required.token, Handler).expect("Error creating client");

    let manager = client.shard_manager.clone();

    {
        let mut data = client.data.lock();
        data.insert::<VoiceManager>(Arc::clone(&client.voice_manager));
    }

    let owners = match http::get_current_application_info() {
        Ok(info) => {
            let mut set = HashSet::new();
            set.insert(info.owner.id);

            set
        }
        Err(why) => panic!("Couldn't get application info: {:?}", why),
    };

    client.with_framework(
        StandardFramework::new()
            .configure(|c| c.owners(owners).prefix(&CONFIG.required.prefix)
                .on_mention(CONFIG.required.mention)) //the trait does not accept a reference
            .before(|_, _m, cmd_name| {
                println!("{:?}", _m);
                println!("Running command {}", cmd_name);
                true
            })
            .after(|_, _m, cmd_name, error| {
                //  Print out an error if it happened
                if let Err(why) = error {
                    println!("Error in {}: {:?}", cmd_name, why);
                } else {
                    let mut duration = Utc::now().signed_duration_since(_m.timestamp);
                    println!("Command '{}' completed in {:#?}", cmd_name, duration);
                }
            })
            .command("ping", |c| c.cmd(commands::meta::ping))
            .command("multiply", |c| c.cmd(commands::math::multiply))
            .command("fib", |c| c.cmd(commands::math::fibonacci))
            .command("uptime", |c| c.cmd(commands::meta::uptime))
            .command("quit", |c| c.cmd(commands::owner::quit).owners_only(true))
            .command("join", |c| c.cmd(commands::voice::join))
            .command("leave", |c| c.cmd(commands::voice::leave))
            .command("play", |c| c.cmd(commands::voice::play))
            .command("mute", |c| c.cmd(commands::voice::mute))
            .command("unmute", |c| c.cmd(commands::voice::unmute))
            .command("deafen", |c| c.cmd(commands::voice::deafen))
            .command("undeafen", |c| c.cmd(commands::voice::undeafen)),
    );

    thread::spawn(move || loop {
        thread::sleep(Duration::from_secs(30));

        let lock = manager.lock();
        let shard_runners = lock.runners.lock();

        for (id, runner) in shard_runners.iter() {
            println!(
                "Shard ID {} is {} with a latency of {:?}",
                id, runner.stage, runner.latency,
            );
        }
    });

    if let Err(why) = client
        .start_shards(SHARDS)
        .map_err(|why| println!("Client ended: {:?}", why))
    {
        println!("Client error: {:?}", why);
    }
}
