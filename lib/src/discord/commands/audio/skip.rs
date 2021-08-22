use serenity::{
    client::Context,
    framework::standard::{macros::command, CommandResult},
    model::channel::Message,
};

use crate::discord::check_msg;
use crate::discord::Lavalink;

/// Skips current track.
#[command]
async fn skip(ctx: &Context, msg: &Message) -> CommandResult {
    let lava_client = {
        let data_read = ctx.data.read().await;
        data_read.get::<Lavalink>().unwrap().clone()
    };

    if let Some(track) = lava_client.skip(msg.guild_id.unwrap()).await {
        check_msg(
            msg.channel_id
                .say(
                    ctx,
                    format!("Skipped: {}", track.track.info.as_ref().unwrap().title),
                )
                .await,
        );
    } else {
        check_msg(msg.channel_id.say(ctx, "Nothing to skip.").await);
    }

    Ok(())
}
