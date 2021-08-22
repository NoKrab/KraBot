use serenity::{
    client::Context,
    framework::standard::{macros::command, CommandResult},
    model::channel::Message,
};

use crate::discord::{check_msg, Lavalink};

/// Shows some information about the current playing track
#[command]
#[aliases(np)]
async fn now_playing(ctx: &Context, msg: &Message) -> CommandResult {
    let lava_client = {
        let data_read = ctx.data.read().await;
        data_read.get::<Lavalink>().unwrap().clone()
    };

    if let Some(node) = lava_client.nodes().await.get(&msg.guild_id.unwrap().0) {
        if let Some(track) = &node.now_playing {
            let requester = if let Some(u) = track.requester {
                u.to_serenity().to_user(ctx).await.unwrap_or_default().name
            } else {
                "Unknown".to_string()
            };

            check_msg(
                msg.channel_id
                    .send_message(&ctx.http, |m| {
                        m.embed(|e| {
                            super::yt_embed(e, track.track.info.as_ref().unwrap(), 1, &requester)
                        });
                        m
                    })
                    .await,
            );
        } else {
            check_msg(
                msg.channel_id
                    .say(&ctx.http, "Nothing is playing at the moment.")
                    .await,
            );
        }
    } else {
        check_msg(
            msg.channel_id
                .say(&ctx.http, "Nothing is playing at the moment.")
                .await,
        );
    }

    Ok(())
}
