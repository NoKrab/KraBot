#[macro_use]
extern crate log;
#[macro_use]
extern crate serde_derive;
#[macro_use]
extern crate serenity;
extern crate toml;
extern crate typemap;

mod config;
mod commands;

use config::Config;
use std::collections::HashSet;
use serenity::framework::StandardFramework;
use serenity::model::event::ResumedEvent;
use serenity::model::gateway::Ready;
use serenity::model::id::ChannelId;
use serenity::model::misc::Mentionable;
use serenity::model::channel::Message;
use serenity::prelude::*;
use serenity::http;
use serenity::client::bridge::voice::ClientVoiceManager;
use serenity::client::{CACHE, Client, Context, EventHandler};
use serenity::Result as SerenityResult;
use serenity::prelude::Mutex;
use serenity::voice;
use std::sync::Arc;
use std::time::SystemTime;
use typemap::Key;

struct VoiceManager;

impl Key for VoiceManager {
    type Value = Arc<Mutex<ClientVoiceManager>>;
}


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
            .configure(|c| c.owners(owners)
                .prefix(&config.required.prefix)
                .on_mention(true))
            .cmd("deafen", deafen)
            .cmd("join", join)
            .cmd("leave", leave)
            .cmd("mute", mute)
            .cmd("play", play)
            .cmd("ping", ping)
            .cmd("undeafen", undeafen)
            .cmd("unmute", unmute));

    let _ = client.start().map_err(|why| println!("Client ended: {:?}", why));
}

command!(deafen(ctx, msg) {
    let guild_id = match CACHE.read().guild_channel(msg.channel_id) {
        Some(channel) => channel.read().guild_id,
        None => {
            check_msg(msg.channel_id.say("Groups and DMs not supported"));

            return Ok(());
        },
    };

    let mut manager_lock = ctx.data.lock().get::<VoiceManager>().cloned().unwrap();
    let mut manager = manager_lock.lock();

    let handler = match manager.get_mut(guild_id) {
        Some(handler) => handler,
        None => {
            check_msg(msg.reply("Not in a voice channel"));

            return Ok(());
        },
    };

    if handler.self_deaf {
        check_msg(msg.channel_id.say("Already deafened"));
    } else {
        handler.deafen(true);

        check_msg(msg.channel_id.say("Deafened"));
    }
});

command!(join(ctx, msg, args) {
    let connect_to = match args.single::<u64>() {
        Ok(id) => ChannelId(id),
        Err(_) => {
            check_msg(msg.reply("Requires a valid voice channel ID be given"));

            return Ok(());
        },
    };

    let guild_id = match CACHE.read().guild_channel(msg.channel_id) {
        Some(channel) => channel.read().guild_id,
        None => {
            check_msg(msg.channel_id.say("Groups and DMs not supported"));

            return Ok(());
        },
    };

    let mut manager_lock = ctx.data.lock().get::<VoiceManager>().cloned().unwrap();
    let mut manager = manager_lock.lock();

    if manager.join(guild_id, connect_to).is_some() {
        check_msg(msg.channel_id.say(&format!("Joined {}", connect_to.mention())));
    } else {
        check_msg(msg.channel_id.say("Error joining the channel"));
    }
});

command!(leave(ctx, msg) {
    let guild_id = match CACHE.read().guild_channel(msg.channel_id) {
        Some(channel) => channel.read().guild_id,
        None => {
            check_msg(msg.channel_id.say("Groups and DMs not supported"));

            return Ok(());
        },
    };

    let mut manager_lock = ctx.data.lock().get::<VoiceManager>().cloned().unwrap();
    let mut manager = manager_lock.lock();
    let has_handler = manager.get(guild_id).is_some();

    if has_handler {
        manager.remove(guild_id);

        check_msg(msg.channel_id.say("Left voice channel"));
    } else {
        check_msg(msg.reply("Not in a voice channel"));
    }
});

command!(mute(ctx, msg) {
    let guild_id = match CACHE.read().guild_channel(msg.channel_id) {
        Some(channel) => channel.read().guild_id,
        None => {
            check_msg(msg.channel_id.say("Groups and DMs not supported"));

            return Ok(());
        },
    };

    let mut manager_lock = ctx.data.lock().get::<VoiceManager>().cloned().unwrap();
    let mut manager = manager_lock.lock();

    let handler = match manager.get_mut(guild_id) {
        Some(handler) => handler,
        None => {
            check_msg(msg.reply("Not in a voice channel"));

            return Ok(());
        },
    };

    if handler.self_mute {
        check_msg(msg.channel_id.say("Already muted"));
    } else {
        handler.mute(true);

        check_msg(msg.channel_id.say("Now muted"));
    }
});

command!(ping(_context, msg) {
    check_msg(msg.channel_id.say("Pong!"));
});

command!(play(ctx, msg, args) {
    let url = match args.single::<String>() {
        Ok(url) => url,
        Err(_) => {
            check_msg(msg.channel_id.say("Must provide a URL to a video or audio"));

            return Ok(());
        },
    };

    if !url.starts_with("http") {
        check_msg(msg.channel_id.say("Must provide a valid URL"));

        return Ok(());
    }

    let guild_id = match CACHE.read().guild_channel(msg.channel_id) {
        Some(channel) => channel.read().guild_id,
        None => {
            check_msg(msg.channel_id.say("Error finding channel info"));

            return Ok(());
        },
    };

    let mut manager_lock = ctx.data.lock().get::<VoiceManager>().cloned().unwrap();
    let mut manager = manager_lock.lock();

    if let Some(handler) = manager.get_mut(guild_id) {
        let source = match voice::ytdl(&url) {
            Ok(source) => source,
            Err(why) => {
                println!("Err starting source: {:?}", why);

                check_msg(msg.channel_id.say("Error sourcing ffmpeg"));

                return Ok(());
            },
        };

        handler.play(source);

        check_msg(msg.channel_id.say("Playing song"));
    } else {
        check_msg(msg.channel_id.say("Not in a voice channel to play in"));
    }
});

command!(undeafen(ctx, msg) {
    let guild_id = match CACHE.read().guild_channel(msg.channel_id) {
        Some(channel) => channel.read().guild_id,
        None => {
            check_msg(msg.channel_id.say("Error finding channel info"));

            return Ok(());
        },
    };

    let mut manager_lock = ctx.data.lock().get::<VoiceManager>().cloned().unwrap();
    let mut manager = manager_lock.lock();

    if let Some(handler) = manager.get_mut(guild_id) {
        handler.deafen(false);

        check_msg(msg.channel_id.say("Undeafened"));
    } else {
        check_msg(msg.channel_id.say("Not in a voice channel to undeafen in"));
    }
});

command!(unmute(ctx, msg) {
    let guild_id = match CACHE.read().guild_channel(msg.channel_id) {
        Some(channel) => channel.read().guild_id,
        None => {
            check_msg(msg.channel_id.say("Error finding channel info"));

            return Ok(());
        },
    };
    let mut manager_lock = ctx.data.lock().get::<VoiceManager>().cloned().unwrap();
    let mut manager = manager_lock.lock();

    if let Some(handler) = manager.get_mut(guild_id) {
        handler.mute(false);

        check_msg(msg.channel_id.say("Unmuted"));
    } else {
        check_msg(msg.channel_id.say("Not in a voice channel to undeafen in"));
    }
});

/// Checks that a message successfully sent; if not, then logs why to stdout.
fn check_msg(result: SerenityResult<Message>) {
    if let Err(why) = result {
        println!("Error sending message: {:?}", why);
    }
}