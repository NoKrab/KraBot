use serenity::{
    builder::CreateEmbed,
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

fn create_embed<'a>(e: &'a mut CreateEmbed, m: &Metadata) -> &'a mut CreateEmbed {
    let v: Vec<&str> = m.git.split('-').collect();
    if v.len() == 2 {
        e.url(format!("https://github.com/NoKrab/KraBot/commit/{}", v[1]));
    };
    e.title("Application Version");
    e.field("version", m.version, true);
    e.field("git commit", m.git, true);
    e.field("compiled", m.date, false);
    e
}

/// Displays the current crate version.
/// Repository: https://github.com/NoKrab/KraBot
#[command]
async fn version(ctx: &Context, msg: &Message) -> CommandResult {
    let metadata = METADATA.lock().await;
    // let me = *metadata;
    check_msg(
        msg.channel_id
            .send_message(&ctx.http, |m| {
                m.embed(|e| create_embed(e, &metadata));
                m
            })
            .await,
    );

    Ok(())
}
