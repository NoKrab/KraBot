use serenity::{
    client::Context,
    framework::standard::{macros::command, CommandResult},
    model::channel::Message,
};

use crate::discord::check_msg;
use crate::discord::Lavalink;

use super::is_playing;

#[command]
#[aliases(unpause)]
async fn resume(ctx: &Context, msg: &Message) -> CommandResult {
    let data = ctx.data.read().await;
    let lava_client = data.get::<Lavalink>().unwrap().clone();
    let guild = msg.guild(&ctx.cache).await.unwrap();
    let guild_id = guild.id;

    if is_playing(&lava_client, guild_id).await {
        check_msg(
            msg.channel_id
                .say(&ctx.http, "Player is already unpaused.")
                .await,
        );
        return Ok(());
    }

    if lava_client.resume(guild_id).await.is_ok() {
        check_msg(msg.channel_id.say(&ctx.http, "Player unpaused.").await);
    }
    Ok(())
}
