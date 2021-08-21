use std::time::Duration;

use serenity::{
    client::Context,
    framework::standard::{macros::command, Args, CommandResult},
    model::channel::Message,
};

use crate::discord::check_msg;
use crate::discord::Lavalink;

#[command]
async fn seek(ctx: &Context, msg: &Message, args: Args) -> CommandResult {
    let time_str = args.message().to_string();
    let data = ctx.data.read().await;
    let lava_client = data.get::<Lavalink>().unwrap().clone();
    let guild = msg.guild(&ctx.cache).await.unwrap();
    let guild_id = guild.id;

    if lava_client
        .nodes()
        .await
        .get(&guild_id.0)
        .and_then(|node| node.now_playing.as_ref().map(|_| ()))
        .is_none()
    {
        check_msg(
            msg.channel_id
                .say(&ctx.http, "Nothing is playing at the moment.")
                .await,
        );
        return Ok(());
    }

    let time_segments = time_str
        .splitn(3, ":")
        .map(|seg| seg.parse::<u64>())
        .collect::<Vec<_>>();

    if time_segments.is_empty() || time_segments.iter().any(|seg| seg.is_err()) {
        check_msg(
            msg.channel_id
                .say(ctx, "Invalid time format. Try hh:mm:ss")
                .await,
        );
        return Ok(());
    }

    let time_segments = time_segments
        .into_iter()
        .map(|seg| seg.unwrap())
        .collect::<Vec<u64>>();

    let time_secs = match time_segments.len() {
        1 => time_segments[0],
        2 => time_segments[0] * 60 + time_segments[1],
        3 => time_segments[2] * 3600 + time_segments[1] * 60 + time_segments[0],
        _ => unreachable!(),
    };

    let time = Duration::from_secs(time_secs);

    if lava_client.seek(guild_id, time).await.is_err() {
        check_msg(msg.channel_id.say(ctx, "Failed to seek").await);
    }

    Ok(())
}
