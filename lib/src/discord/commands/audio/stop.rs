use serenity::{
    client::Context,
    framework::standard::{macros::command, CommandResult},
    model::channel::Message,
};

use crate::discord::{check_msg, Lavalink};

/// Stops the current player and clears the queue.
#[command]
async fn stop(ctx: &Context, msg: &Message) -> CommandResult {
    let lava_client = {
        let data_read = ctx.data.read().await;
        data_read.get::<Lavalink>().unwrap().clone()
    };

    let guild = msg.guild(&ctx.cache).await.unwrap();
    let guild_id = guild.id;

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
