#[macro_use]
extern crate log;
#[macro_use]
extern crate serde_derive;
#[macro_use]
extern crate serenity;
extern crate toml;

mod config;
mod commands;

use config::Config;
use std::collections::HashSet;
use serenity::framework::StandardFramework;
use serenity::model::event::ResumedEvent;
use serenity::model::gateway::Ready;
use serenity::model::id::ChannelId;
use serenity::model::misc::Mentionable;
use serenity::prelude::*;
use serenity::http;
use serenity::client::bridge::voice::ClientVoiceManager;
use serenity::client::{CACHE, Client, Context, EventHandler};
use serenity::Result as SerenityResult;
use std::sync::Arc;
use std::time::SystemTime;

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
    Config::hello();
    println!("{:?}", config);
    /*assert_eq!(config.required.token, "ibimseintoken");
    assert_eq!(config.optional.setting, true);*/
    assert_eq!(config.required.prefix, "!");

    let mut client = Client::new(&config.required.token, Handler).expect("Error creating client");

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
//                let sys_time = SystemTime::now();
//                println!("{:?}", &_sys_time);

                println!("Running command {}", cmd_name);
                true
            })
            .after(|_, _m, cmd_name, error| {
                //  Print out an error if it happened
                if let Err(why) = error {
                    println!("Error in {}: {:?}", cmd_name, why);
                } else {
                    println!("Command executed: {}", cmd_name);
//                    let difference = _sys_time.duration_since(_sys_time)
//                                            .expect("SystemTime::duration_since failed");
//                    println!("{:?}", difference);
                }
            })
            .command("ping", |c| c.cmd(commands::meta::ping))
            .command("multiply", |c| c.cmd(commands::math::multiply))
            .command("fib", |c| c.cmd(commands::math::fibonacci))
            .command("uptime", |c| c.cmd(commands::meta::uptime))
            .command("quit", |c| c.cmd(commands::owner::quit).owners_only(true)),
    );

    if let Err(why) = client.start() {
        println!("Client error: {:?}", why);
    }
}
