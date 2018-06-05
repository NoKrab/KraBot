use serde_json::Value;
use util::api::imgur::imgur::*;
use serenity::client::CACHE;
use serenity::Result as SerenityResult;
use serenity::model::channel::{Channel, Message};
use regex::Regex;
use rand;
use rand::Rng;

lazy_static! {
    static ref RE_TAGS: Regex = Regex::new(r"(#[a-zA-Z]+\b)").unwrap();
}

command!(get_imgs(_ctx, msg, args) {
    if let Some(result) = Account::account_images() {
        let items = match result.as_array() {
                Some(array) => array.to_owned(),
                None => Vec::new(),
        };
        debug!("Imgs: {:?}", items);
        check_msg(msg.channel_id.send_message(|m| {           
            m.embed(|e| {
                let mut e = e;
                e = e.title("Imgur Images");
                for img in &items {
                    e = e.field(&img["name"].as_str().unwrap(), format!("Link: {}\nId: {}",&img["link"].as_str().unwrap(), &img["id"].as_str().unwrap()), false);
                }
                e
            })
        }));
    } else {
        check_msg(msg.channel_id.say("Failed to retrieve images"));
    }
});

command!(get_albums(_ctx, msg, args) {
 if let Some(result) = Account::albums() {
        let albums = match result.as_array() {
                Some(array) => array.to_owned(),
                None => Vec::new(),
        };
        check_msg(msg.channel_id.send_message(|m| {
            m.embed(|e| {
                let mut e = e;
                    e = e.title("Imgur Albums");
                    for album in &albums {
                        e = e.field(&album["title"].as_str().unwrap(), format!("Link: {}\nId: {}",&album["link"].as_str().unwrap(), &album["id"].as_str().unwrap()), false);
                    }
                    e
                })
        }));
    } else {
        check_msg(msg.channel_id.say("Failed to retrieve albums"));
    }
});

command!(get_current_album(_ctx, msg) {
    let guild_id = match CACHE.read().guild_channel(msg.channel_id) {
        Some(channel) => channel.read().guild_id,
        None => {
            check_msg(msg.channel_id.say("Error finding channel info"));
            return Ok(());
        },
    };
    if let Ok(album_id) = get_current_album_id(guild_id.to_string().parse::<u64>().unwrap() as i64) {
        check_msg(msg.channel_id.say(album_id));
    } else {
        check_msg(msg.channel_id.say("there was a problem reading the album_id"));
    }
});

command!(set_album(_ctx, msg, args) {
    debug!("Channel ID: {}", msg.channel_id);
    debug!("{:?}", msg.guild_id());
    let guild_id = match CACHE.read().guild_channel(msg.channel_id) {
        Some(channel) => channel.read().guild_id,
        None => {
            check_msg(msg.channel_id.say("Error finding channel info"));
            return Ok(());
        },
    };
    let album_id = match args.single::<String>() {
        Ok(album_id) => album_id,
        Err(_) => {
            check_msg(msg.reply("Requires a valid album ID"));
            return Ok(());
        },
    };

    if let Err(e) = set_album_id(&album_id, guild_id.to_string().parse::<u64>().unwrap() as i64) {
        check_msg(msg.channel_id.say("oops could not insert new album_id into database"));
    } else {
        check_msg(msg.channel_id.say("Updated album!"));
    }

});

command!(query_img(_ctx, msg, args) {
    //get album imgs
    let guild_id = match CACHE.read().guild_channel(msg.channel_id) {
        Some(channel) => channel.read().guild_id,
        None => {
            check_msg(msg.channel_id.say("Error finding channel info"));
            return Ok(());
        },
    };

     if let Some(result) = Album::album_images(guild_id.to_string().parse::<u64>().unwrap() as i64) {
        let images = match result.as_array() {
                Some(array) => array.to_owned(),
                None => Vec::new(),
        };
        let mut best_match_count: i32 = -1;
        let mut matched_imgs: Vec<usize> = Vec::new();
        let args: Vec<&str> = args.full().split(' ').collect(); //Splits args of msg e.g. "a b" into ["a", "b"]

        for (i, img) in images.iter().enumerate() {
            let description = &img["description"].as_str().unwrap();
            let mut match_count: i32 = -1;
            for mat in RE_TAGS.find_iter(description) {
                for arg in &args {
                    let tag: String = mat.as_str().to_owned().replace("#", "");
                    if &tag == arg {
                        match_count = match_count + 1;
                    }
                }
            }
            debug!("Matched: {}", match_count + 1);
            if match_count > best_match_count {
                matched_imgs.clear();
                matched_imgs.push(i);
                best_match_count = match_count;
            } else if match_count > -1 && match_count == best_match_count {
                matched_imgs.push(i);
            }
        }
        debug!("Result: {:?}", &matched_imgs);
        debug!("Result Length: {}", matched_imgs.len());
        debug!("Best match count: {}", best_match_count);
        if best_match_count > -1 {
            match matched_imgs.len() {
                1 => check_msg(msg.channel_id.say(&images[matched_imgs[0]]["link"])),
                _ => {
                    let mut rng = rand::thread_rng();
                    let number = rng.gen_range(0, matched_imgs.len());
                    debug!("RNG: {}", number);
                    check_msg(msg.channel_id.say(&images[matched_imgs[number]]["link"].as_str().unwrap()))
                }
            }
        } else {
            check_msg(msg.channel_id.say("No matching image found :("));
        }
    } else {
        check_msg(msg.channel_id.say("Failed to retrieve albums"));
    }
});

/// Checks that a message successfully sent; if not, then logs why to stdout.
fn check_msg(result: SerenityResult<Message>) {
    if let Err(why) = result {
        println!("Error sending message: {:?}", why);
    }
}