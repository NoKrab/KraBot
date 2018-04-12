use util::api::twitch::twitch::API;
use serenity::Result as SerenityResult;
use serenity::model::channel::Message;


command!(twitch(_ctx, msg, args) {
    let code = args.single::<String>().unwrap();
    if let Some(emote_uri) = API::get_emote_uri(&code) {
        check_msg(msg.channel_id.say(&emote_uri));
    } else {
        check_msg(msg.channel_id.say("Emote not found!"));
    }
});

///------------------------------------Functions------------------------------------

/// Checks that a message successfully sent; if not, then logs why to stdout.
fn check_msg(result: SerenityResult<Message>) {
    if let Err(why) = result {
        println!("Error sending message: {:?}", why);
    }
}