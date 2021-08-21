use serenity::{
    client::Context,
    framework::standard::{macros::command, CommandResult},
    model::channel::Message,
};

use crate::discord::Lavalink;
use crate::{discord::check_msg, env::get_bot_prefix};

use super::is_playing;

#[command]
async fn pause(ctx: &Context, msg: &Message) -> CommandResult {
    let data = ctx.data.read().await;
    let lava_client = data.get::<Lavalink>().unwrap().clone();
    let guild = msg.guild(&ctx.cache).await.unwrap();
    let guild_id = guild.id;

    if is_playing(&lava_client, guild_id).await {
        check_msg(
            msg.channel_id
                .say(&ctx.http, "Nothing is playing at the moment.")
                .await,
        );
        return Ok(());
    }

    if lava_client.pause(guild_id).await.is_ok() {
        check_msg(
            msg.channel_id
                .say(
                    &ctx.http,
                    format!(
                        "Player paused. Use command `{}resume` to unpause the player.",
                        get_bot_prefix()
                    ),
                )
                .await,
        );
    }
    Ok(())
}
