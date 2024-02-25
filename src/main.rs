#![allow(non_snake_case)]
#[macro_use]
extern crate tracing;

use dotenvy::dotenv;
use std::env;

mod cli;
mod commands;
mod voice;

use cli::setup_cli;

use voice::music_advanced;
use voice::music_basic;
use voice::music_events;

use lavalink_rs::{model::events, prelude::*};

use poise::serenity_prelude as serenity;
use songbird::SerenityInit;

pub struct Data {
    pub lavalink: LavalinkClient,
}

pub type Error = Box<dyn std::error::Error + Send + Sync>;
pub type Context<'a> = poise::Context<'a, Data, Error>;

#[tokio::main]
async fn main() -> Result<(), Error> {
    setup_cli();
    dotenv().expect(".env file not found");

    let framework = poise::Framework::builder()
        .options(poise::FrameworkOptions {
            commands: vec![
                commands::general::ping(),
                commands::general::twobee(),
                commands::fun::roll(),
                music_basic::play(),
                music_basic::join(),
                music_basic::leave(),
                music_advanced::queue(),
                music_advanced::skip(),
                music_advanced::pause(),
                music_advanced::resume(),
                music_advanced::stop(),
                music_advanced::seek(),
                music_advanced::clear(),
                music_advanced::remove(),
                music_advanced::swap(),
                music_advanced::volume(),
            ],
            prefix_options: poise::PrefixFrameworkOptions {
                prefix: Some(
                    env::var("BOT_PREFIX")
                        .expect("BOT_PREFIX not set")
                        .to_string(),
                ),
                ..Default::default()
            },
            ..Default::default()
        })
        .setup(|ctx, _ready, framework| {
            Box::pin(async move {
                poise::builtins::register_globally(ctx, &framework.options().commands).await?;

                let events = events::Events {
                    raw: Some(music_events::raw_event),
                    ready: Some(music_events::ready_event),
                    track_start: Some(music_events::track_start),
                    ..Default::default()
                };

                let node_local = NodeBuilder {
                    hostname: env::var("LAVALINK_HOSTNAME")
                        .expect("LAVALINK_HOSTNAME not set")
                        .to_string(),
                    is_ssl: false,
                    events: events::Events::default(),
                    password: env::var("LAVALINK_PASSWORD")
                        .expect("LAVALINK_PASSWORD not set")
                        .to_string(),
                    user_id: ctx.cache.current_user().id.into(),
                    session_id: None,
                };

                let client = LavalinkClient::new(
                    events,
                    vec![node_local],
                    NodeDistributionStrategy::round_robin(),
                )
                .await;

                Ok(Data { lavalink: client })
            })
        })
        .build();

    let mut client = serenity::ClientBuilder::new(
        env::var("DISCORD_TOKEN").expect("missing DISCORD_TOKEN"),
        serenity::GatewayIntents::all(),
    )
    .register_songbird()
    .framework(framework)
    .await?;

    client.start().await?;

    Ok(())
}
