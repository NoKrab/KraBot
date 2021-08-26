use crate::discord::check_msg;
use passwords::PasswordGenerator;
use serenity::{
    client::Context,
    framework::standard::{macros::command, CommandResult},
    model::channel::Message,
};

/// generate a password, but actually don't use it, since it's posted in a public environment
#[command]
async fn password(context: &Context, msg: &Message) -> CommandResult {
    let pg = PasswordGenerator {
        length: 32,
        numbers: true,
        lowercase_letters: true,
        uppercase_letters: true,
        symbols: true,
        spaces: false,
        exclude_similar_characters: true,
        strict: true,
    };
    check_msg(
        msg.channel_id
            .say(&context.http, pg.generate_one().unwrap())
            .await,
    );

    Ok(())
}
