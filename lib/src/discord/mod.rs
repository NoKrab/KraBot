use core::panic;
use std::time::Duration;

mod commands;

use commands::audio::join::*;
use commands::audio::leave::*;
use commands::audio::now_playing::*;
use commands::audio::play::*;
use commands::audio::queue::*;
use commands::audio::skip::*;

use commands::general::ping::*;

use super::env::{get_bot_prefix, get_discord_token, get_lavalink_env};

use serenity::{
    async_trait,
    client::{Client, Context, EventHandler},
    framework::{
        standard::{
            macros::{group, hook},
            CommandResult,
        },
        StandardFramework,
    },
    http::Http,
    model::{channel::Message, gateway::Ready, id::GuildId},
    Result as SerenityResult,
};

use lavalink_rs::{gateway::*, model::*, LavalinkClient};
use serenity::prelude::*;
use songbird::SerenityInit;
use tokio::time::sleep;

struct Lavalink;

impl TypeMapKey for Lavalink {
    type Value = LavalinkClient;
}

struct Handler;
struct LavalinkHandler;

#[async_trait]
impl EventHandler for Handler {
    async fn ready(&self, _: Context, ready: Ready) {
        info!("{} is connected!", ready.user.name);
    }

    async fn cache_ready(&self, _: Context, guilds: Vec<GuildId>) {
        info!("cache is ready!\n{:#?}", guilds);
    }
}

#[async_trait]
impl LavalinkEventHandler for LavalinkHandler {
    async fn track_start(&self, _client: LavalinkClient, event: TrackStart) {
        info!("Track started!\nGuild: {}", event.guild_id);
    }
    async fn track_finish(&self, _client: LavalinkClient, event: TrackFinish) {
        info!("Track finished!\nGuild: {}", event.guild_id);
    }
}

#[hook]
async fn after(_ctx: &Context, _msg: &Message, command_name: &str, command_result: CommandResult) {
    if let Err(e) = command_result {
        error!("Command '{}' returned error {:?} => {}", command_name, e, e)
    }
}

#[group]
#[only_in(guilds)]
#[commands(join, leave, play, now_playing, skip, ping, queue)]
struct General;

pub async fn start() -> Result<(), Box<dyn std::error::Error>> {
    let token = get_discord_token();

    let http = Http::new_with_token(&token);

    let bot_id = match http.get_current_application_info().await {
        Ok(info) => info.id,
        Err(why) => panic!("Could not access application info: {:?}", why),
    };

    let framework = StandardFramework::new()
        .configure(|c| c.prefix(&get_bot_prefix()))
        .after(after)
        .group(&GENERAL_GROUP);

    let mut client = Client::builder(&token)
        .event_handler(Handler)
        .framework(framework)
        .register_songbird()
        .await
        .expect("Err creating client");

    let (host, port, auth) = get_lavalink_env();

    let mut remaining_attempts = 20;
    let lava_client = loop {
        if let Ok(client) = LavalinkClient::builder(bot_id)
            .set_host(&host)
            .set_port(port)
            .set_password(&auth)
            .build(LavalinkHandler)
            .await
        {
            info!("Connected to LavaLink Server");
            break client;
        }

        remaining_attempts -= 1;

        if remaining_attempts < 0 {
            error!("Could not connect to LavaLink Server. Is it running?");
            std::process::exit(0);
        }

        // Give LavaLink some time to boot...
        sleep(Duration::from_millis(2000)).await;
    };

    {
        let mut data = client.data.write().await;
        data.insert::<Lavalink>(lava_client);
    }

    let _ = client
        .start()
        .await
        .map_err(|why| error!("Client ended: {:?}", why));

    Ok(())
}

fn check_msg(result: SerenityResult<Message>) {
    if let Err(why) = result {
        error!("Error sending message: {:?}", why);
    }
}
