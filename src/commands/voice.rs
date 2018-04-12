use CONFIG;
use serde_json;
use std::mem;
use serenity::model::id::UserId;
use std::sync::Arc;
use std::str;
use std::io;
use futures::{Future, Stream};
use hyper::Client;
use hyper_tls::HttpsConnector;
use tokio_core::reactor::Core;
use hyper::{Method, Request};
use serde_json::Value;
use typemap::Key;
use serenity::client::bridge::voice::ClientVoiceManager;
use serenity::client::CACHE;
use serenity::model::id::ChannelId;
use serenity::model::channel::Message;
use serenity::voice;
use serenity::prelude::*;
use serenity::prelude::Mutex;
use serenity::Result as SerenityResult;
use serenity::utils::Colour;
use util::api::youtube::youtube::API;
use regex::Regex;

pub struct VoiceManager;

impl Key for VoiceManager {
    type Value = Arc<Mutex<ClientVoiceManager>>;
}

lazy_static! {
    static ref RE_PLAY: Regex = Regex::new(r"^http.+|^[1-5]").unwrap();
}

command!(stop(ctx, msg) {
    let guild_id = match CACHE.read().guild_channel(msg.channel_id) {
        Some(channel) => channel.read().guild_id,
        None => {
            check_msg(msg.channel_id.say("Groups and DMs not supported"));

            return Ok(());
        },
    };
    let mut manager_lock = ctx.data.lock().get::<VoiceManager>().cloned().unwrap();
    let mut manager = manager_lock.lock();

    if let Some(handler) = manager.get_mut(guild_id) {
        handler.stop();

        check_msg(msg.channel_id.say("Audioplayer stopped."));
    } else {
        check_msg(msg.channel_id.say("Not in a voice channel to stop player"));
    }
});

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
    let guild_id = match CACHE.read().guild_channel(msg.channel_id) {
        Some(channel) => channel.read().guild_id,
        None => {
            check_msg(msg.channel_id.say("Groups and DMs not supported"));

            return Ok(());
        },
    };

    //Gets HashMap from all Users in the current guild which are in a Voice Channel: (UserID ->VoiceState)
    let voice_members = match CACHE.read().guild(guild_id) {
        Some(guild) => {
            guild.read().voice_states.clone()
        },
        None => {
            return Ok(());
        },
    };
     println!("User in Voice: {:#?}", voice_members);

    //If User is in Voice join VoiceChannel...
    if voice_members.contains_key(&msg.author.id) {             //Searches for Key (UserId) in HashMap
        check_msg(msg.reply("Author is in Voice channel"));
        let connect_to = match voice_members.get(&msg.author.id) {
            Some(voice_state) => voice_state.channel_id.unwrap(),
            None => {
                return Ok(());
            },
        };
        println!("Voice Channel: {:?}", connect_to);

        let mut manager_lock = ctx.data.lock().get::<VoiceManager>().cloned().unwrap();
        let mut manager = manager_lock.lock();

        if manager.join(guild_id, connect_to).is_some() {
            check_msg(msg.channel_id.say(&format!("Joined {}", connect_to.mention())));
        } else {
            check_msg(msg.channel_id.say("Error joining the channel"));
        }
    } else { //... If not use args
       check_msg(msg.reply("Author ist not in Voice Channel"));
        let connect_to = match args.single::<u64>() {
            Ok(id) => ChannelId(id),
            Err(_) => {
                check_msg(msg.reply("Requires a valid voice channel ID be given"));

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

command!(play(ctx, msg, args) {
    let args = args.full();
    if RE_PLAY.is_match(&args) {
        let url = get_url(&args, &msg.author.id);

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
             let response = format!(
                    "Playing: {}",
                    url
                );
            check_msg(msg.channel_id.say(&response));
        } else {
            check_msg(msg.channel_id.say("Not in a voice channel to play in"));
        }
    } else if !args.is_empty() {
        let query = str::replace(args, " ", "+");
        debug!("Query: {}", query);
        API::youtube_search(query, msg);
    } else {
        check_msg(msg.channel_id.say("Must provide a URL, Query or Selection [1-5]"));
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

command!(search(_ctx, msg, args) {
    let query = str::replace(args.full(), " ", "+");
    debug!("{}", query);
    API::youtube_search(query, msg);
});

///------------------------------------Functions------------------------------------

/// Checks that a message successfully sent; if not, then logs why to stdout.
fn check_msg(result: SerenityResult<Message>) {
    if let Err(why) = result {
        println!("Error sending message: {:?}", why);
    }
}

///Checks if play-command parameter is URL or Number
fn get_url(s: &str, user_id: &UserId) -> String {
    let re = Regex::new(r"^[1-5]").unwrap();
    if re.is_match(s)  {
        return API::get_url(user_id, s.parse::<usize>().unwrap() - 1); //-1 because indexing starts at 0
    } else {
        return String::from(s);
    }
}

