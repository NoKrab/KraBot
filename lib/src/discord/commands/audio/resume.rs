use serenity::{
    client::Context,
    framework::standard::{macros::command, CommandResult},
    model::channel::Message,
};

use crate::discord::Lavalink;

#[command]
#[aliases(unpause)]
async fn resume(ctx: &Context, msg: &Message) -> CommandResult {
    let lava_client = {
        let data_read = ctx.data.read().await;
        data_read.get::<Lavalink>().unwrap().clone()
    };

    lava_client.resume(msg.guild_id.unwrap()).await?;

    Ok(())
}
