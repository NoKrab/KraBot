extern crate transient_hashmap;

use std::io::{self, Write};
use self::transient_hashmap::TransientHashMap;
use std::sync::Mutex;
use serenity::http;
use std::mem;
use serenity::model::id::UserId;

lazy_static! {
    static ref SAVES: Mutex<TransientHashMap<UserId, &'static str>> = Mutex::new(TransientHashMap::new(5)); //u32, &'static str>
}

fn string_to_static_str(s: String) -> &'static str {
    unsafe { //auweia
        let ret = mem::transmute(&s as &str);
        mem::forget(s);
        ret
    }
}

command!(quit(ctx, msg, _args) {
    ctx.quit();

    let _ = msg.reply("Shutting down!");
});

command!(save(ctx, msg, args) {
    let mut to_save = match args.single::<String>() {
        Ok(to_save) => {
            let s: &'static str = string_to_static_str(to_save);
            SAVES.lock().unwrap().insert(msg.author.id, s);
        },
        Err(_) => {
            msg.reply("Nothing to save :thinking:");
            return Ok(());
        }
    };
});

command!(load(ctx, msg, args) {
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

command!(host(ctx, msg, _args) {
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

command!(clear(_ctx, msg, _args) {
    let channel_id = msg.channel_id.0;
    debug!("ChannelID: {:#?}", channel_id);
//    let channel_num = channel_id as u64;
    let _messages = match  http::get_messages(channel_id, &"/search?content=clear") {
        Ok(msgs) =>  {
            info!("Messages: {:#?}", msgs);
            msgs
        },
        Err(_) => {
            error!("No messages found!");
            return Ok(());
        }
    };

});
