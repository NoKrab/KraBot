use serenity::{
    client::Context,
    framework::standard::{macros::command, CommandResult},
    model::channel::Message,
};

use crate::discord::{check_msg, Lavalink};

#[command]
async fn stop(ctx: &Context, msg: &Message) -> CommandResult {
    let guild = msg.guild(&ctx.cache).await.unwrap();
    let guild_id = guild.id;

    let data = ctx.data.read().await;
    let lava_client = data.get::<Lavalink>().unwrap().clone();

    if lava_client.stop(guild_id).await.is_ok() {
        if let Some(mut node) = lava_client.nodes().await.get_mut(&msg.guild_id.unwrap().0) {
            node.now_playing = None;
            node.queue.clear();
        }

        check_msg(
            msg.reply(ctx, "Player stopped.\nQueue emptied. :wastebasket:")
                .await,
        );
    } else {
        check_msg(msg.reply(ctx, "Failed to stop player").await);
    }

    Ok(())
}
