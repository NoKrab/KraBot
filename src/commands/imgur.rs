use serde_json::Value;
use util::api::imgur::imgur::{Account};


command!(get_imgs(_ctx, msg, args) {
    if let Some(result) = Account::account_images() {
        let items = match result.as_array() {
                Some(array) => array.to_owned(),
                None => Vec::new(),
        };
        let _ = msg.channel_id.send_message(|mut m| {
            m.content("Imgur Images");
            m.embed(|mut e| {
            for img in &items {
                e.field(&img["name"].as_str().unwrap(), &img["link"].as_str().unwrap(), false);
            }
            e
            });
            m
        });
    } else {
        let _ = msg.channel_id.say("?????");
    }
});