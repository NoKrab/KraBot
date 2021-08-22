use serenity::{
    client::Context,
    framework::standard::{macros::command, Args, CommandResult},
    model::channel::Message,
};

use crate::discord::{check_msg, Lavalink};

/// Adds an entire playlist to the queue.
///
/// Usage: `playlist https://www.youtube.com/playlist?list=PLTktV6LgA75yif8RR7yUiSttZD7GKtl_5`
#[command]
#[min_args(1)]
#[aliases(playlist, playplaylist, play_list, pl)]
async fn play_playlist(ctx: &Context, msg: &Message, args: Args) -> CommandResult {
    let query = args.message().to_string();

    let channel_res = ctx.cache.guild_channel(msg.channel_id).await;
    if channel_res.is_none() {
        msg.channel_id
            .say(ctx, "Error finding channel info")
            .await?;
        return Ok(());
    }

    let guild_id = channel_res.unwrap().guild_id;

    let manager = songbird::get(ctx).await.unwrap().clone();

    if let Some(_handler_lock) = manager.get(guild_id) {
        let lava_client = {
            let data_read = ctx.data.read().await;
            data_read.get::<Lavalink>().unwrap().clone()
        };

        let mut iter = 0;
        let query_information = loop {
            iter += 1;
            let res = lava_client.auto_search_tracks(&query).await?;

            if res.tracks.is_empty() {
                if iter == 5 {
                    msg.channel_id
                        .say(&ctx, "Could not find any video of the search query.")
                        .await?;
                    return Ok(());
                }
            } else {
                break res;
            }
        };

        for track in &query_information.tracks {
            lava_client
                .play(guild_id, track.clone())
                .requester(msg.author.id)
                .queue()
                .await?;
        }

        msg.channel_id
            .send_message(ctx, |m| {
                m.content(format!(
                    "Added {} songs to queue.",
                    query_information.tracks.len()
                ));
                m.embed(|e| {
                    e.title("Playlist link");
                    e.url(query);
                    e.footer(|f| f.text(format!("Submited by {}", &msg.author.name)))
                })
            })
            .await?;
    } else {
        check_msg(msg.channel_id.say(ctx, "Please, connect the bot to the voice channel you are currently on first with the `join` command.").await);
    }

    Ok(())
}
