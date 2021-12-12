use crate::discord::check_msg;
use rand::{thread_rng, Rng};
use serenity::{
    client::Context,
    framework::standard::{macros::command, Args, CommandResult},
    model::channel::Message,
};

/// Rolls a dice.
/// Usage:
/// - `roll` (Default: 1-6)
/// - `roll 100` (1-100)
/// - `roll 5-20`
#[command]
#[aliases(dice, rng)]
#[delimiters("-", " ")]
async fn roll(ctx: &Context, msg: &Message, mut args: Args) -> CommandResult {
    let rolled = match args.len() {
        0 => thread_rng().gen_range(1..=6u32),
        1 => thread_rng().gen_range(1..=args.single()?),
        2 => thread_rng().gen_range(args.single()?..=args.single()?),
        _ => {
            check_msg(
                msg.channel_id
                    .say(&ctx.http, "Wrong number of arguments")
                    .await,
            );
            return Ok(());
        }
    };
    check_msg(msg.channel_id.say(&ctx.http, rolled).await);
    Ok(())
}
