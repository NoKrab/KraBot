use serenity::{
    client::Context,
    framework::standard::{macros::command, CommandResult},
    futures::lock::Mutex,
    model::channel::Message,
};

use crate::discord::check_msg;
use crate::misc::Metadata;

lazy_static! {
    pub static ref METADATA: Mutex<Metadata> = Mutex::new(Metadata::default());
}

pub async fn set_metadata(metadata: Metadata) {
    let mut mut_metadata = METADATA.lock().await;
    *mut_metadata = metadata;
}

/// Displays the current crate version.
/// Repository: https://github.com/NoKrab/KraBot
#[command]
async fn version(context: &Context, msg: &Message) -> CommandResult {
    let metadata = METADATA.lock().await;
    check_msg(
        msg.channel_id
            .say(&context.http, format!("{}", metadata.version))
            .await,
    );

    Ok(())
}
