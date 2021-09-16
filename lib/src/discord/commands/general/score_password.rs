use crate::discord::check_msg;
use passwords::{analyzer, scorer};
use serenity::{
    client::Context,
    framework::standard::{macros::command, Args, CommandResult},
    model::channel::Message,
};

/// Score your password:
/// A password whose score is,
///
///     0 ~ 20 is very dangerous (may be cracked within few seconds)
///     20 ~ 40 is dangerous
///     40 ~ 60 is very weak
///     60 ~ 80 is weak
///     80 ~ 90 is good
///     90 ~ 95 is strong
///     95 ~ 99 is very strong
///     99 ~ 100 is invulnerable
///
/// Don't actually input your actual password tho?
#[command]
#[aliases(sp)]
async fn score_password(context: &Context, msg: &Message, args: Args) -> CommandResult {
    check_msg(
        msg.channel_id
            .say(
                &context.http,
                scorer::score(&analyzer::analyze(args.message().to_owned())),
            )
            .await,
    );

    Ok(())
}
