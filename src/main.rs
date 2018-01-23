#[macro_use]
extern crate serde_derive;
extern crate serenity;
extern crate toml;

mod config;

use config::Config;
use serenity::prelude::*;
use serenity::model::channel::Message;
use serenity::model::gateway::Ready;

struct Handler;

impl EventHandler for Handler {
    fn message(&self, _: Context, msg: Message) {
        if msg.content == "!ping" {
            if let Err(why) = msg.channel_id.say("Pong!") {
                println!("Error sending message: {:?}", why);
            }
        } else if msg.content == "!messageme" {
            if let Err(why) = msg.author.dm(|m| m.content("Hello!")) {
                println!("Error when direct messaging user: {:?}", why);
            }
        }
    }

    fn ready(&self, _: Context, ready: Ready) {
        println!("{} is connected!", ready.user.name);
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

    if let Err(why) = client.start() {
        println!("Client error: {:?}", why);
    }

    /*if let Err(why) = client.start_shards(2) {
        println!("Client error {:?}", why);
    }*/
}
