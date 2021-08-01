use std::time::Duration;

use serenity::{
    client::Context,
    framework::standard::{macros::command, Args, CommandResult},
    model::channel::{Message, ReactionType},
};

use crate::{
    discord::{check_msg, Lavalink},
    env::get_bot_prefix,
};

#[command]
#[min_args(1)]
async fn play(ctx: &Context, msg: &Message, args: Args) -> CommandResult {
    let query = args.message().to_string();

    let guild_id = match ctx.cache.guild_channel(msg.channel_id).await {
        Some(channel) => channel.guild_id,
        None => {
            check_msg(
                msg.channel_id
                    .say(&ctx.http, "Error finding channel info")
                    .await,
            );

            return Ok(());
        }
    };

    let manager = songbird::get(ctx).await.unwrap().clone();

    if let Some(_handler) = manager.get(guild_id) {
        let data = ctx.data.read().await;
        let lava_client = data.get::<Lavalink>().unwrap().clone();

        let query_information = lava_client.auto_search_tracks(&query).await?;
        let tracks = query_information.tracks;

        if tracks.is_empty() {
            check_msg(
                msg.channel_id
                    .say(&ctx, "Could not find any video of the search query.")
                    .await,
            );
            return Ok(());
        }

        let mut track_idx = 0;
        let emojis = ["1️⃣", "2️⃣", "3️⃣", "4️⃣", "5️⃣"];
        let reactions = emojis
            .iter()
            .map(|emoji| emoji.parse().unwrap())
            .collect::<Vec<ReactionType>>();

        if tracks.len() > 1 {
            if let Ok(react_msg) = msg
                .channel_id
                .send_message(&ctx.http, |m| {
                    m.embed(|e| {
                        e.title("🎵 Tracks found 🎵");
                        for (idx, track) in tracks.iter().take(5).enumerate() {
                            if let Some(ref info) = track.info {
                                e.field(
                                    format!("{} {}", emojis[idx], info.title),
                                    &info.uri,
                                    false,
                                );
                            } else {
                                e.field(emojis[idx], "No track info", false);
                            }
                        }

                        e
                    });
                    m.reactions(
                        reactions
                            .into_iter()
                            .take(tracks.len())
                            .collect::<Vec<ReactionType>>(),
                    );

                    m
                })
                .await
            {
                if let Some(reaction) = &react_msg
                    .await_reaction(&ctx)
                    .timeout(Duration::from_secs(10))
                    .author_id(msg.author.id)
                    .await
                {
                    let emoji = &reaction.as_inner_ref().emoji;
                    match emoji.as_data().as_str() {
                        "1️⃣" => track_idx = 0,
                        "2️⃣" => track_idx = 1,
                        "3️⃣" => track_idx = 2,
                        "4️⃣" => track_idx = 3,
                        "5️⃣" => track_idx = 4,
                        _ => {
                            check_msg(
                                msg.channel_id
                                    .say(&ctx.http, "Wrong reaction. You had one job.")
                                    .await,
                            );

                            return Ok(());
                        }
                    }
                } else {
                    check_msg(msg.channel_id.say(&ctx.http, "Nothing selected").await);

                    return Ok(());
                }
            }
        }

        if let Err(why) = &lava_client
            .play(guild_id, tracks[track_idx].clone())
            .queue()
            .await
        {
            error!("{}", why);
            return Ok(());
        };
        check_msg(
            msg.channel_id
                .say(
                    &ctx.http,
                    format!(
                        "Added to queue: {}",
                        tracks[track_idx].info.as_ref().unwrap().title
                    ),
                )
                .await,
        );
        if let Some(track_info) = &tracks[track_idx].info {
            let queue_len = lava_client
                .nodes()
                .await
                .get(&msg.guild_id.unwrap().0)
                .unwrap()
                .queue
                .len();

            check_msg(
                msg.channel_id
                    .send_message(&ctx.http, |m| {
                        m.content("Added to queue");
                        m.embed(|e| super::yt_embed(e, track_info, queue_len));
                        m
                    })
                    .await,
            );
        }
    } else {
        check_msg(
            msg.channel_id
                .say(
                    &ctx.http,
                    format!(
                        "Use `{}join` first, to connect the bot to your current voice channel.",
                        get_bot_prefix()
                    ),
                )
                .await,
        );
    }

    Ok(())
}
