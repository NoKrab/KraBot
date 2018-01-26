#[macro_use]
extern crate log;
#[macro_use]
extern crate serde_derive;
#[macro_use]
extern crate serenity;
extern crate toml;
extern crate typemap;
extern crate chrono;

mod config;
mod commands;

use std::sync::Arc;
use std::collections::HashSet;
use std::fs;
use std::fs::File;
use std::io::Write;
use config::Config;
use std::path::Path;
use serenity::framework::StandardFramework;
use serenity::model::event::ResumedEvent;
use serenity::model::gateway::Ready;
use serenity::prelude::*;
use serenity::http;
use chrono::prelude::*;
use commands::voice::VoiceManager;

struct Handler;

impl EventHandler for Handler {
    fn ready(&self, ctx: Context, ready: Ready) {
        ctx.set_game_name("I bims ein Rust Bot");
        println!("{} is connected!", ready.user.name);
        //this is actually a terrible idea
        if !Path::new("./log").exists() { fs::create_dir("./log").expect("Error creating folder") };
        let mut file = File::create("./log/startuptime.log").expect("Error creating file!");
        file.write_fmt(format_args!("{:?}", Utc::now())).expect("Error writing to file!");
    }

    fn resume(&self, _: Context, _: ResumedEvent) {
        info!("Resumed");
    }
}

fn main() {
    let config = Config::read_config("./config/config.toml");
    println!("{:?}", config);

    let mut client = Client::new(&config.required.token, Handler).expect("Error creating client");

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
            .configure(|c| c.owners(owners).prefix(&config.required.prefix)
                .on_mention(config.required.mention)) //the trait does not accept a reference
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

    let _ = client.start().map_err(|why| println!("Client ended: {:?}", why));
}