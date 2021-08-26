use crate::discord::check_msg;
use passwords::{analyzer, scorer, PasswordGenerator};
use serenity::{
    builder::CreateEmbed,
    client::Context,
    framework::standard::{macros::command, CommandResult},
    model::channel::Message,
};

fn create_embed<'a>(e: &'a mut CreateEmbed) -> &'a mut CreateEmbed {
    let pg = PasswordGenerator {
        length: 32,
        numbers: true,
        lowercase_letters: true,
        uppercase_letters: true,
        symbols: true,
        spaces: false,
        exclude_similar_characters: true,
        strict: true,
    };
    let p = pg.generate_one().unwrap();
    let s = scorer::score(&analyzer::analyze(&p));
    e.title("Password Generator");
    e.url("https://crates.io/crates/passwords");
    e.field("password", p, true);
    e.field("score", s, true);
    e
}

/// generate a password, but actually don't use it, since it's posted in a public environment
/// also shows score, table is shown @help "sp"
#[command]
#[aliases(pg)]
async fn password(ctx: &Context, msg: &Message) -> CommandResult {
    check_msg(
        msg.channel_id
            .send_message(&ctx.http, |m| {
                m.embed(|e| create_embed(e));
                m
            })
            .await,
    );

    Ok(())
}
