use std::time::Duration;

use serenity::{
    client::Context,
    framework::standard::{macros::command, Args, CommandResult},
    model::channel::Message,
};

use crate::discord::check_msg;
use crate::discord::Lavalink;

/// Jumps to a position in the current track. Expects a timestamp of format `hh:mm:ss`.
///
/// Usage:
/// - `seek 2:50:30` Jumps to 02:50:30
/// - `seek 50:30` Jumps 00:50:30
/// - `seek 30` Jumps to 00:00:30
#[command]
#[min_args(1)]
#[aliases(jump)]
async fn seek(ctx: &Context, msg: &Message, args: Args) -> CommandResult {
    let lava_client = {
        let data_read = ctx.data.read().await;
        data_read.get::<Lavalink>().unwrap().clone()
    };
    let time_str = args.message().to_string();

    let guild = msg.guild(&ctx.cache).await.unwrap();
    let guild_id = guild.id;

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

    lava_client.seek(guild_id, time).await?;

    Ok(())
}
