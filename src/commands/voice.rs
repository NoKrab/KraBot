use CONFIG;
use serde_json;
use std::mem;
use serenity::model::id::UserId;
use transient_hashmap::TransientHashMap;
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

pub struct VoiceManager;

impl Key for VoiceManager {
    type Value = Arc<Mutex<ClientVoiceManager>>;
}

lazy_static! {
//u32, &'static str>
    static ref LINKS: Mutex<TransientHashMap<UserId, &'static str>> = Mutex::new(TransientHashMap::new(5));
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

command!(search(_ctx, msg, _args) {
    let mut core = Core::new()?;
    let handle = core.handle();
    let client = Client::configure()
        .connector(HttpsConnector::new(4, &handle)?)
        .build(&handle);

    let uri = "https://www.googleapis.com/youtube/v3/search?part=snippet&q=lul&maxResults=5&key=AIzaSyC0YMET9zKtfyF6npTQRheyDR2wL3SYDCY".parse()?;
    let mut req = Request::new(Method::Get, uri);

    let post = client.request(req).and_then(|res| {
        println!("GET: {}", res.status());

        res.body().concat2().and_then(move |body| {
             let v: Value = serde_json::from_slice(&body).map_err(|e| {
            io::Error::new(
                io::ErrorKind::Other,
                e
            )
        })?;
        Ok(v)
        })
    });
    let work = post;
    let posted = core.run(work).unwrap().to_string();
    debug!("GET: {:?}", &posted);
    let serde: Value = serde_json::from_str(&posted)?;

    debug!("GET: {:?}", &serde);

    let s: &'static str = string_to_static_str(posted);
    let mut links = LINKS.lock();
    links.insert(msg.author.id, s);
    info!("{:?}", links.get(&msg.author.id).unwrap());
    let out = links.get(&msg.author.id).unwrap();
    let out_serde: Value = serde_json::from_str(&out)?;

//    let _ = msg.channel_id.send_message(&out_serde["items"][0]["snippet"]["thumbnails"]["default"]["url"].to_string());
    let thumb_url = &out_serde["items"][0]["snippet"]["thumbnails"]["default"]["url"].to_string().replace("\"", "");

    if let Err(why) = msg.channel_id.send_message(|m| m
                .content("WoW")
                .embed(|e| e
                    .title("Youtube Results")
                    .description("This is a description")
//                    .thumbnail(|t| t.url(&thumb_url))
                    .colour(Colour::dark_teal())
                    .footer(|f| f
                        .icon_url(&thumb_url)
                        .text("Some text")))) {
                println!("Error sending message: {:?}", why);
            }
});

command!(test(_ctx, msg, args) {
    let query = str::replace(args.full(), " ", "+");
    debug!("{}", query);
    youtube_search(query);
//    let _ = msg.channel_id.say(query);
});

/// Checks that a message successfully sent; if not, then logs why to stdout.
fn check_msg(result: SerenityResult<Message>) {
    if let Err(why) = result {
        println!("Error sending message: {:?}", why);
    }
}

fn string_to_static_str(s: String) -> &'static str {
    unsafe {
        let ret = mem::transmute(&s as &str);
        error!("AUWEIA {}", ret);
        mem::forget(s);
        ret
    }
}

fn youtube_search(query: String) {
    let token = match CONFIG.optional.youtube_token {
        Some(ref token) => token.to_owned(),
        None => panic!("no token"), // TODO allow "empty" token
    };
    let limit = 5;
    let mut core = Core::new().unwrap();
    let handle = core.handle();
    let client = Client::configure()
        .connector(HttpsConnector::new(4, &handle).unwrap())
        .build(&handle);

    let uri = format!(
        "https://www.googleapis.com/youtube/v3/search?part=snippet&q={}&maxResults={}&key={}",
        query, limit, token
    );
    let uri = &uri[..];
    debug!("{}", uri);

    let request = client
        .request(Request::new(Method::Get, uri.parse().unwrap()))
        .and_then(|res| {
            debug!("GET: {}", res.status());

            res.body().concat2().and_then(move |body| {
                let v: Value = serde_json::from_slice(&body)
                    .map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;
                Ok(v)
            })
        });

    let result = core.run(request).unwrap();

    let items = match result["items"].as_array() {
        Some(array) => array.to_owned(),
        None => Vec::new(),
    };

    //    let items = result["items"].as_array();
    //    debug!("{:#?}", result);
    //    debug!("{}", result["etag"]);
    //    debug!("{}", result["items"]);
    debug!("{}", items.len());
    debug!("{:#?}", items);
}
