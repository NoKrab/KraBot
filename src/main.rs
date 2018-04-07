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
extern crate tokio_core;
extern crate toml;
extern crate transient_hashmap;
extern crate typemap;
extern crate regex;

mod config;
mod commands;
mod database;
mod util;

use config::Config;
use database::sqlite::sqlite;
use commands::voice::VoiceManager;
//use util::network;

use std::fs;
use std::sync::Arc;
use std::collections::HashSet;
use std::collections::HashMap;
// use std::fs::File;
// use std::io::Write;
use std::path::Path;
use chrono::prelude::*;
use serenity::prelude::*;
use serenity::prelude::Mutex;
use serenity::framework::StandardFramework;
use serenity::framework::standard::DispatchError;
use serenity::model::event::ResumedEvent;
use serenity::model::gateway::Ready;
use serenity::client::bridge::gateway::ShardManager;
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
        ctx.set_game_name("I do not work :)");
        if let Some(shard) = ready.shard {
            // Note that array index 0 is 0-indexed, while index 1 is 1-indexed.
            //
            // This may seem unintuitive, but it models Discord's behaviour.
            info!(
                "{} is connected on shard {}/{}!",
                ready.user.name, shard[0], shard[1],
            );
            let con = sqlite::create_connection(&*SQLITE_PATH);
            sqlite::create_bot_table(&con);
            sqlite::insert_timestamp(&con, shard[0] as i64, ready.user.name);
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

impl Key for CommandCounter {
    type Value = HashMap<String, u64>;
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

    let mut client = Client::new(&*CONFIG.required.token, Handler).expect("Error creating client");

    // let manager = client.shard_manager.clone();

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
        Err(why) => {
            error!("Couldn't get application info: {}", why);
            panic!()
        }
    };

    client.with_framework(
        StandardFramework::new()
            .configure(|c| {
                c.owners(owners)
                    .prefix(&*CONFIG.required.prefix)
                    .on_mention(CONFIG.required.mention)
                    .delimiters(vec![", ", ","])
            })
            .before(|ctx, _m, cmd_name| {
                debug!("{:?}", _m);
                debug!("Running command {}", cmd_name);
                let mut data = ctx.data.lock();
                let counter = data.get_mut::<CommandCounter>().unwrap();
                let entry = counter.entry(cmd_name.to_string()).or_insert(0);
                *entry += 1;
                true
            })
            .after(|_, _m, cmd_name, error| match error {
                Ok(()) => {
                    let mut duration = Utc::now().signed_duration_since(_m.timestamp);
                    debug!("Command '{}' completed in {:#?}", cmd_name, duration);
                }
                Err(why) => error!("Command '{}' returned error {:?}", cmd_name, why),
            })
            .on_dispatch_error(|_ctx, msg, error| {
                if let DispatchError::RateLimited(seconds) = error {
                    let _ = msg.channel_id
                        .say(&format!("Try this again in {} seconds", seconds));
                }
            })
            .command("ping", |c| {
                c.cmd(commands::meta::ping)
            })
            .command("multiply", |c| {
                c.cmd(commands::math::multiply)
            })
            .command("fib", |c| {
                c.cmd(commands::math::fibonacci)
            })
            .command("uptime", |c| {
                c.cmd(commands::meta::uptime)
            })
            .command("quit", |c| c.cmd(commands::owner::quit).owners_only(true))
            .command("clear", |c| c.cmd(commands::owner::clear).owners_only(true))
            .command("host", |c| c.cmd(commands::owner::host).owners_only(true))
            .command("save", |c| c.cmd(commands::owner::save).owners_only(true))
            .command("load", |c| c.cmd(commands::owner::load).owners_only(true))
            .group("Voice", |g| {
                g.command("join", |c| {
                    c.cmd(commands::voice::join)
                }).command("leave", |c| {
                        c.cmd(commands::voice::leave)
                    })
                    .command("play", |c| {
                        c.cmd(commands::voice::play)
                    })
                    .command("mute", |c| {
                        c.cmd(commands::voice::mute)
                    })
                    .command("unmute", |c| {
                        c.cmd(commands::voice::unmute)
                    })
                    .command("deafen", |c| {
                        c.cmd(commands::voice::deafen)
                    })
                    .command("undeafen", |c| {
                        c.cmd(commands::voice::undeafen)
                    })
                    .command("search", |c| {
                        c.cmd(commands::voice::search)
                    })
                    .command("test", |c| {
                        c.cmd(commands::voice::test)
                    })
            })
            .command("commands", |c| {
                c.cmd(commands::meta::commands)
            }),
    );

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

    if let Err(why) = client
        .start_shards(CONFIG.required.shards)
        .map_err(|why| error!("Client ended: {:?}", why))
    {
        error!("Client error: {:?}", why);
    }
}

fn setup_logger() -> Result<(), fern::InitError> {
    if !Path::new("./log").exists() {
        fs::create_dir("./log").expect("Error creating folder")
    };

    let file_config = fern::Dispatch::new()
        .level(log::LevelFilter::Error)
        .chain(fern::log_file("./log/rust.log")?);

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
        .level(log::LevelFilter::Debug)
        .chain(std::io::stdout());

    stdout_config.chain(file_config).apply()?;

    debug!("Debug output enabled.");
    error!("Error output enabled.");
    info!("Info output enabled.");
    Ok(())
}
