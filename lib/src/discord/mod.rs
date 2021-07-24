use super::env::{get_bot_prefix, get_discord_token, get_lavalink_env};

use serenity::{
    async_trait,
    client::{Client, Context, EventHandler},
    framework::{
        standard::{
            macros::{command, group, hook},
            Args, CommandResult,
        },
        StandardFramework,
    },
    http::Http,
    model::{channel::Message, gateway::Ready, id::GuildId, misc::Mentionable},
    Result as SerenityResult,
};

use lavalink_rs::{gateway::*, model::*, LavalinkClient};
use serenity::prelude::*;
use songbird::SerenityInit;

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
#[commands(join, leave, play, now_playing, skip, ping)]
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
    let lava_client = LavalinkClient::builder(bot_id)
        .set_host(host)
        .set_port(port)
        .set_password(auth)
        .build(LavalinkHandler)
        .await?;

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

#[command]
async fn join(ctx: &Context, msg: &Message) -> CommandResult {
    let guild = msg.guild(&ctx.cache).await.unwrap();
    let guild_id = guild.id;

    let channel_id = guild
        .voice_states
        .get(&msg.author.id)
        .and_then(|voice_state| voice_state.channel_id);

    let connect_to = match channel_id {
        Some(channel) => channel,
        None => {
            check_msg(msg.reply(ctx, "Join a voice channel.").await);

            return Ok(());
        }
    };

    let manager = songbird::get(ctx).await.unwrap().clone();

    let (_, handler) = manager.join_gateway(guild_id, connect_to).await;

    match handler {
        Ok(connection_info) => {
            let data = ctx.data.read().await;
            let lava_client = data.get::<Lavalink>().unwrap().clone();
            lava_client.create_session(&connection_info).await?;

            check_msg(
                msg.channel_id
                    .say(&ctx.http, &format!("Joined {}", connect_to.mention()))
                    .await,
            );
        }
        Err(why) => check_msg(
            msg.channel_id
                .say(&ctx.http, format!("Error joining the channel: {}", why))
                .await,
        ),
    }

    Ok(())
}

#[command]
async fn leave(ctx: &Context, msg: &Message) -> CommandResult {
    let guild = msg.guild(&ctx.cache).await.unwrap();
    let guild_id = guild.id;

    let manager = songbird::get(ctx).await.unwrap().clone();
    let has_handler = manager.get(guild_id).is_some();

    if has_handler {
        if let Err(e) = manager.remove(guild_id).await {
            check_msg(
                msg.channel_id
                    .say(&ctx.http, format!("Failed: {:?}", e))
                    .await,
            );
        }

        {
            let data = ctx.data.read().await;
            let lava_client = data.get::<Lavalink>().unwrap().clone();
            lava_client.destroy(guild_id).await?;
        }

        check_msg(msg.channel_id.say(&ctx.http, "Left voice channel").await);
    } else {
        check_msg(msg.reply(ctx, "Not in a voice channel").await);
    }

    Ok(())
}

#[command]
async fn ping(context: &Context, msg: &Message) -> CommandResult {
    check_msg(msg.channel_id.say(&context.http, "Pong!").await);

    Ok(())
}

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

        if query_information.tracks.is_empty() {
            check_msg(
                msg.channel_id
                    .say(&ctx, "Could not find any video of the search query.")
                    .await,
            );
            return Ok(());
        }

        if let Err(why) = &lava_client
            .play(guild_id, query_information.tracks[0].clone())
            // Change this to play() if you want your own custom queue or no queue at all.
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
                        query_information.tracks[0].info.as_ref().unwrap().title
                    ),
                )
                .await,
        );
    } else {
        check_msg(
            msg.channel_id
                .say(
                    &ctx.http,
                    "Use `~join` first, to connect the bot to your current voice channel.",
                )
                .await,
        );
    }

    Ok(())
}

#[command]
#[aliases(np)]
async fn now_playing(ctx: &Context, msg: &Message) -> CommandResult {
    let data = ctx.data.read().await;
    let lava_client = data.get::<Lavalink>().unwrap().clone();

    if let Some(node) = lava_client.nodes().await.get(&msg.guild_id.unwrap().0) {
        if let Some(track) = &node.now_playing {
            check_msg(
                msg.channel_id
                    .say(
                        &ctx.http,
                        format!("Now Playing: {}", track.track.info.as_ref().unwrap().title),
                    )
                    .await,
            );
        } else {
            check_msg(
                msg.channel_id
                    .say(&ctx.http, "Nothing is playing at the moment.")
                    .await,
            );
        }
    } else {
        check_msg(
            msg.channel_id
                .say(&ctx.http, "Nothing is playing at the moment.")
                .await,
        );
    }

    Ok(())
}

#[command]
async fn skip(ctx: &Context, msg: &Message) -> CommandResult {
    let data = ctx.data.read().await;
    let lava_client = data.get::<Lavalink>().unwrap().clone();

    if let Some(track) = lava_client.skip(msg.guild_id.unwrap()).await {
        check_msg(
            msg.channel_id
                .say(
                    ctx,
                    format!("Skipped: {}", track.track.info.as_ref().unwrap().title),
                )
                .await,
        );
    } else {
        check_msg(msg.channel_id.say(ctx, "Nothing to skip.").await);
    }

    Ok(())
}

fn check_msg(result: SerenityResult<Message>) {
    if let Err(why) = result {
        error!("Error sending message: {:?}", why);
    }
}
