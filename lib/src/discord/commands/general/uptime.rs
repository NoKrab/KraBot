use serenity::{
    client::Context,
    framework::standard::{macros::command, CommandResult},
    model::channel::Message,
};

use crate::discord::{check_msg, global_data::Uptime};

/// Displays the bot uptime.
#[command]
async fn uptime(ctx: &Context, msg: &Message) -> CommandResult {
    let uptime = {
        let instant = {
            let data_read = ctx.data.read().await;
            data_read.get::<Uptime>().unwrap().clone()
        };

        let duration = instant.elapsed();
        let seconds = duration.as_secs();

        let days = seconds / 60 / 60 / 24;
        let hours = seconds / 3600 % 24;
        let minutes = seconds % 3600 / 60;
        let sec = seconds % 3600 % 60;

        if days == 0 {
            format!("{}:{:02}:{:02}", hours, minutes, sec)
        } else {
            format!("{}D {}:{:02}:{:02}", days, hours, minutes, sec)
        }
    };

    check_msg(
        msg.channel_id
            .say(&ctx.http, format!("Uptime: {}", uptime))
            .await,
    );

    Ok(())
}
