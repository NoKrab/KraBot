extern crate chrono;
#[macro_use]
extern crate lazy_static;
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

mod config;
mod commands;
mod database;

use config::Config;
use database::sqlite::sqlite;
use commands::voice::VoiceManager;

use std::sync::Arc;
use std::collections::HashSet;
use std::collections::HashMap;
use std::thread;
// use std::fs;
// use std::fs::File;
// use std::io::Write;
// use std::path::Path;
use std::time::Duration;
use chrono::prelude::*;
use serenity::prelude::*;
use serenity::prelude::Mutex;
use serenity::framework::StandardFramework;
use serenity::framework::standard::{help_commands, Args, CommandOptions, DispatchError,
                                    HelpBehaviour};
use serenity::model::Permissions;
use serenity::model::event::ResumedEvent;
use serenity::model::gateway::Ready;
use serenity::model::channel::Message;
use serenity::client::bridge::gateway::{ShardId, ShardManager};
use serenity::http;
use typemap::Key;

// What actual use does this bring?
lazy_static! {
    static ref CONFIG: Config = Config::get_config(config::CONFIG_PATH);
    static ref SQLITE_PATH: (String, String) = Config::get_sqlite_path(config::CONFIG_PATH);
}

struct Handler;

struct CommandCounter;

struct ShardManagerContainer;

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
            let _ = sqlite::get_timestamp(con);
            //            let _ = con.close().expect("Failed to close connection");
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

impl Key for CommandCounter {
    type Value = HashMap<String, u64>;
}

impl Key for ShardManagerContainer {
    type Value = Arc<Mutex<ShardManager>>;
}

fn main() {
    //    let config = Config::get_config(config::CONFIG_PATH);
    //    println!("{:?}", config);
    println!("Configuration file: {:?}", *CONFIG);
    println!("SQLITE PATH: {:?}", *SQLITE_PATH);

    let mut client = Client::new(&*CONFIG.required.token, Handler).expect("Error creating client");

    let manager = client.shard_manager.clone();

    {
        let mut data = client.data.lock();
        data.insert::<CommandCounter>(HashMap::default());
        data.insert::<ShardManagerContainer>(Arc::clone(&client.shard_manager));
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

    let phramework = StandardFramework::new();

    client.with_framework(
        phramework
            .configure(|c| {
                c.owners(owners)
                    .prefix(&*CONFIG.required.prefix)
                    .on_mention(CONFIG.required.mention)
                    .delimiters(vec![", ", ","])
            })
            .before(|ctx, _m, cmd_name| {
                println!("{:?}", _m);
                println!("Running command {}", cmd_name);
                let mut data = ctx.data.lock();
                let counter = data.get_mut::<CommandCounter>().unwrap();
                let entry = counter.entry(cmd_name.to_string()).or_insert(0);
                *entry += 1;
                true
            })
            .after(|_, _m, cmd_name, error| match error {
                Ok(()) => {
                    let mut duration = Utc::now().signed_duration_since(_m.timestamp);
                    println!("Command '{}' completed in {:#?}", cmd_name, duration);
                }
                Err(why) => println!("Command '{}' returned error {:?}", cmd_name, why),
            })
            .on_dispatch_error(|_ctx, msg, error| {
                if let DispatchError::RateLimited(seconds) = error {
                    let _ = msg.channel_id
                        .say(&format!("Try this again in {} seconds", seconds));
                }
            })
            .command("ping", |c| c.cmd(commands::meta::ping))
            .command("multiply", |c| c.cmd(commands::math::multiply))
            .command("fib", |c| c.cmd(commands::math::fibonacci))
            .command("uptime", |c| c.cmd(commands::meta::uptime))
            .command("quit", |c| c.cmd(commands::owner::quit).owners_only(true))
            .command("clear", |c| c.cmd(commands::owner::clear).owners_only(true))
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
        .start_shards(CONFIG.required.shards)
        .map_err(|why| println!("Client ended: {:?}", why))
    {
        println!("Client error: {:?}", why);
    }
}
