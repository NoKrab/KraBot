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

use config::Config;
use std::collections::HashSet;
use serenity::framework::StandardFramework;
use serenity::model::event::ResumedEvent;
use serenity::model::gateway::Ready;
use serenity::prelude::*;
use serenity::http;
use std::sync::Arc;
use std::time::SystemTime;
use chrono::prelude::*;

use commands::voice::VoiceManager;

struct Handler;

impl EventHandler for Handler {
    fn ready(&self, ctx: Context, ready: Ready) {
        ctx.set_game_name("I bims ein Rust Bot");
        println!("{} is connected!", ready.user.name);
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
            .configure(|c| c.owners(owners).prefix(&config.required.prefix))
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
            .command("join", |c| c.cmd(commands::voice::join)),
    );


    let _ = client.start().map_err(|why| println!("Client ended: {:?}", why));
}