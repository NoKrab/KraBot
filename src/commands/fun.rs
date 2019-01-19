use rand;
use rand::Rng;
use serenity::model::channel::Message;
use serenity::Result as SerenityResult;
use util::api::twitch::twitch::API;

command!(twitch(_ctx, msg, args) {
    let code = args.single::<String>().unwrap();
    if let Some(emote_uri) = API::get_emote_uri(&code) {
        check_msg(msg.channel_id.say(&emote_uri));
    } else {
        check_msg(msg.channel_id.say("Emote not found!"));
    }
});

command!(flip(_ctx, msg, _args) {
    let mut rng = rand::thread_rng();
    let paths = match  rng.gen_range(0, 2) {
        0 => vec!["assets/coin_head.png"],
        1 => vec!["assets/coin_tail.png"],
        _ => vec![]
    };

    check_msg( msg.channel_id.send_files(paths, |m| m.content("Rolling...")));
});

/// Checks that a message successfully sent; if not, then logs why to stdout.
fn check_msg(result: SerenityResult<Message>) {
    if let Err(why) = result {
        println!("Error sending message: {:?}", why);
    }
}
