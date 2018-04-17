use serenity::http;
use serenity::model::id::UserId;
use serde_json::*;
use regex::Regex;
use std::mem;
use transient_hashmap::TransientHashMap;
use std::sync::Mutex;


lazy_static! {
    //u32, &'static str>
    static ref SAVES: Mutex<TransientHashMap<UserId, &'static str>> = Mutex::new(TransientHashMap::new(5));
    static ref RE_CLEAR: Regex = Regex::new(r"^[1-5]").unwrap();
}

fn string_to_static_str(s: String) -> &'static str {
    unsafe {
        //auweia
        let ret = mem::transmute(&s as &str);
        mem::forget(s);
        ret
    }
}

command!(quit(ctx, msg, _args) {
    ctx.quit();

    let _ = msg.reply("Shutting down!");
});

command!(save(_ctx, msg, args) {
    let _ = match args.single::<String>() {
        Ok(to_save) => {
            let s: &'static str = string_to_static_str(to_save);
            SAVES.lock().unwrap().insert(msg.author.id, s);
        },
        Err(_) => {
            let _ = msg.reply("Nothing to save :thinking:");
            return Ok(());
        }
    };
});

command!(load(_ctx, msg, _args) {
       SAVES.lock().unwrap().prune();
        // Maybe the item is still there or maybe not  ¯\_(ツ)_/¯
       if SAVES.lock().unwrap().contains_key(&msg.author.id) {
            let _ = msg.reply(SAVES.lock().unwrap().get(&msg.author.id).unwrap());
            let remaining_lifetime = SAVES.lock().unwrap().remaining_lifetime(&msg.author.id).unwrap().to_string();
            let _ = msg.reply(&remaining_lifetime);
       } else {
            let _ = msg.reply("It's already gone ¯\\_(ツ)_/¯");
       }

});

command!(host(_ctx, _msg, _args) {
//    let mut core = Core::new()?;
//    let client = Client::new(&core.handle());
//
//    let uri = "http://httpbin.org/ip".parse()?;
//    let work = client.get(uri).and_then(|res| {
//        println!("Response: {}", res.status());
//
//        res.body().for_each(|chunk| {
//            io::stdout()
//                .write_all(&chunk)
//                .map_err(From::from)
//        })
//    });
//    core.run(work)?;
//    let _ = msg.reply("...");
});

/// refer to https://discordapp.com/developers/docs/resources/channel#get-channel-messages
command!(clear(_ctx, msg, args) {
    let channel_id = msg.channel_id.0;
    let amount = match args.single::<String>() {
        Ok(amount) => amount,
        Err(_) => {
            let _ = msg.reply("Command requires parameter between 1 and 5");
            return Ok(());
        }
    };

    if RE_CLEAR.is_match(&amount) {
        let query = format!("?limit={}", amount);
        let messages = match http::get_messages(channel_id, &query) { //&"/search?content=clear"
            Ok(msgs) =>  {
                debug!("Messages: {:#?}", msgs);
                msgs
            },
            Err(_) => {
                error!("No messages found!");
                return Ok(());
            }
        };
        for message in &messages {
            message.delete();
        }
    } else {
        let _ = msg.reply("Parameter has to be between 1 and 5");
    }


});
