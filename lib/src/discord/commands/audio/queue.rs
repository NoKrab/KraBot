use serenity::{
    client::Context,
    framework::standard::{macros::command, CommandResult},
    model::channel::Message,
};

use crate::discord::{check_msg, Lavalink};

#[command]
async fn queue(ctx: &Context, msg: &Message) -> CommandResult {
    let lava_client = {
        let data_read = ctx.data.read().await;
        data_read.get::<Lavalink>().unwrap().clone()
    };

    if let Some(node) = lava_client.nodes().await.get(&msg.guild_id.unwrap().0) {
        let tracks = node
            .queue
            .iter()
            .map(|track_queue| track_queue.track.info.clone())
            .collect::<Vec<_>>();

        if tracks.is_empty() {
            check_msg(msg.channel_id.say(ctx, "Queue is empty.").await);
            return Ok(());
        }

        check_msg(
            msg.channel_id
                .send_message(&ctx.http, |m| {
                    m.embed(|e| {
                        e.title("Queue");
                        for (idx, track_info) in tracks.iter().take(10).enumerate() {
                            if let Some(info) = track_info {
                                e.field(format!("{}. {}", idx + 1, info.title), &info.uri, false);
                            } else {
                                e.field(idx + 1, "No track info", false);
                            }
                        }

                        e
                    });

                    m
                })
                .await,
        );
    }
    Ok(())
}
