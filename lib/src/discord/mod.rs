use super::env::{get_bot_prefix, get_discord_token, get_lavalink_env};
use futures::stream::StreamExt;
use hyper::{
    client::{Client as HyperClient, HttpConnector},
    Body, Request,
};
use std::net::{SocketAddr, ToSocketAddrs};
use std::{error::Error, future::Future};
use twilight_cache_inmemory::{InMemoryCache, ResourceType};
use twilight_command_parser::{Command, CommandParserConfig, Parser};
use twilight_gateway::{
    cluster::{Cluster, ShardScheme},
    Event,
};
use twilight_http::Client as HttpClient;
use twilight_lavalink::{
    http::LoadedTracks,
    model::{Destroy, Pause, Play, Seek, Stop, Volume},
    Lavalink,
};
use twilight_model::gateway::Intents;
use twilight_model::{channel::Message, gateway::payload::MessageCreate, id::ChannelId};
use twilight_standby::Standby;

#[derive(Clone, Debug)]
struct State {
    http: HttpClient,
    lavalink: Lavalink,
    hyper: HyperClient<HttpConnector>,
    standby: Standby,
    cluster: Cluster,
}

fn spawn(
    fut: impl Future<Output = Result<(), Box<dyn Error + Send + Sync + 'static>>> + Send + 'static,
) {
    tokio::spawn(async move {
        if let Err(why) = fut.await {
            debug!("handler error: {:?}", why);
        }
    });
}

pub async fn start() -> Result<(), Box<dyn Error + Send + Sync + 'static>> {
    let (lhost, lauth) = get_lavalink_env();
    let vlhost: Vec<SocketAddr> = lhost.to_socket_addrs()?.collect();
    debug!("{:#?}", vlhost);
    let lavalink_host = vlhost[0];
    let scheme = ShardScheme::Auto;
    let (cluster, mut events) = Cluster::builder(
        get_discord_token(),
        Intents::GUILD_MESSAGES | Intents::GUILD_VOICE_STATES,
    )
    .shard_scheme(scheme)
    .build()
    .await?;
    let cluster_spawn = cluster.clone();

    tokio::spawn(async move {
        cluster_spawn.up().await;
    });

    let http = HttpClient::new(get_discord_token());
    let user_id = http.current_user().await?.id;
    let lavalink = Lavalink::new(user_id, cluster.shards().len() as u64);
    lavalink.add(lavalink_host, lauth).await?;
    let cache = InMemoryCache::builder()
        .resource_types(ResourceType::MESSAGE)
        .build();
    let state = State {
        http: http.clone(),
        lavalink,
        hyper: HyperClient::new(),
        standby: Standby::new(),
        cluster: cluster.clone(),
    };

    while let Some((shard_id, event)) = events.next().await {
        cache.update(&event);
        state.standby.process(&event);
        state.lavalink.process(&event).await?;
        tokio::spawn(handle_event(shard_id, event, state.clone()));
    }

    Ok(())
}

async fn handle_event(
    shard_id: u64,
    event: Event,
    state: State,
) -> Result<(), Box<dyn Error + Send + Sync>> {
    match event {
        Event::MessageCreate(msg) => {
            let parser = setup_parser();
            match parser.parse(&msg.content) {
                Some(Command { name: "ping", .. }) => spawn(ping(msg.0, shard_id, state)),
                Some(Command { name: "join", .. }) => spawn(join(msg.0, shard_id, state)),
                Some(Command { name: "leave", .. }) => spawn(leave(msg.0, shard_id, state)),
                Some(Command { name: "play", .. }) => spawn(play(msg.0, state)),
                Some(Command { name: "pause", .. }) => spawn(pause(msg.0, state)),
                Some(Command { name: "seek", .. }) => spawn(seek(msg.0, state)),
                Some(Command { name: "stop", .. }) => spawn(stop(msg.0, state)),
                Some(Command { name: "volume", .. }) => spawn(volume(msg.0, state)),
                _ => trace!("Message didn't match a prefix and command"),
            }
        }
        Event::ShardConnected(_) => {
            info!("Connected on shard {}", shard_id);
        }
        _ => {}
    }

    Ok(())
}

fn setup_parser<'a>() -> Parser<'a> {
    let mut config = CommandParserConfig::new();
    config.add_command("ping", false);
    config.add_command("join", false);
    config.add_command("leave", false);
    config.add_command("play", false);
    config.add_command("pause", false);
    config.add_command("seek", false);
    config.add_command("stop", false);
    config.add_command("volume", false);
    config.add_prefix(get_bot_prefix());
    Parser::new(config)
}

async fn ping(
    msg: Message,
    shard_id: u64,
    state: State,
) -> Result<(), Box<dyn Error + Send + Sync + 'static>> {
    info!("Shard ID: {}", shard_id);
    state
        .http
        .create_message(msg.channel_id)
        .content("Pong!")?
        .await?;
    Ok(())
}

async fn join(
    msg: Message,
    shard_id: u64,
    state: State,
) -> Result<(), Box<dyn Error + Send + Sync + 'static>> {
    state
        .http
        .create_message(msg.channel_id)
        .content("What's the channel ID you want me to join?")?
        .await?;

    let author_id = msg.author.id;
    let msg = state
        .standby
        .wait_for_message(msg.channel_id, move |new_msg: &MessageCreate| {
            new_msg.author.id == author_id
        })
        .await?;
    let channel_id = msg.content.parse::<u64>()?;

    if let Some(shard) = state.cluster.shard(shard_id) {
        shard
            .command(&serde_json::json!({
                "op": 4,
                "d": {
                    "channel_id": channel_id,
                    "guild_id": msg.guild_id,
                    "self_mute": false,
                    "self_deaf": false,
                }
            }))
            .await?;

        state
            .http
            .create_message(msg.channel_id)
            .content(format!("Joined <#{}>!", channel_id))?
            .await?;
    } else {
        state
            .http
            .create_message(msg.channel_id)
            .content("Failed joining channel")?
            .await?;
    }

    Ok(())
}

async fn leave(
    msg: Message,
    shard_id: u64,
    state: State,
) -> Result<(), Box<dyn Error + Send + Sync + 'static>> {
    debug!(
        "leave command in channel {} by {}",
        msg.channel_id, msg.author.name
    );

    let guild_id = msg.guild_id.unwrap();
    let player = state.lavalink.player(guild_id).await.unwrap();
    player.send(Destroy::from(guild_id))?;
    if let Some(shard) = state.cluster.shard(shard_id) {
        shard
            .command(&serde_json::json!({
                "op": 4,
                "d": {
                    "channel_id": None::<ChannelId>,
                    "guild_id": msg.guild_id,
                    "self_mute": false,
                    "self_deaf": false,
                }
            }))
            .await?;
    }
    state
        .http
        .create_message(msg.channel_id)
        .content("Left the channel")?
        .await?;

    Ok(())
}

async fn play(msg: Message, state: State) -> Result<(), Box<dyn Error + Send + Sync + 'static>> {
    debug!(
        "play command in channel {} by {}",
        msg.channel_id, msg.author.name
    );
    debug!("{}", msg.content);
    let msg_split: Vec<&str> = msg.content.split(' ').collect();
    if msg_split.len() == 1 {
        state
            .http
            .create_message(msg.channel_id)
            .content("What's the URL of the audio to play?")?
            .await?;

        let author_id = msg.author.id;
        let msg = state
            .standby
            .wait_for_message(msg.channel_id, move |new_msg: &MessageCreate| {
                new_msg.author.id == author_id
            })
            .await?;
        let guild_id = msg.guild_id.unwrap();

        let player = state.lavalink.player(guild_id).await.unwrap();
        let (parts, body) = twilight_lavalink::http::load_track(
            player.node().config().address,
            &msg.content,
            &player.node().config().authorization,
        )?
        .into_parts();
        let req = Request::from_parts(parts, Body::from(body));
        let res = state.hyper.request(req).await?;
        let response_bytes = hyper::body::to_bytes(res.into_body()).await?;

        let loaded = serde_json::from_slice::<LoadedTracks>(&response_bytes)?;

        if let Some(track) = loaded.tracks.first() {
            player.send(Play::from((guild_id, &track.track)))?;

            let content = format!(
                "Playing **{:?}** by **{:?}**",
                track.info.title, track.info.author
            );
            state
                .http
                .create_message(msg.channel_id)
                .content(content)?
                .await?;
        } else {
            state
                .http
                .create_message(msg.channel_id)
                .content("Didn't find any results")?
                .await?;
        }
    } else if msg_split.len() == 2 {
        let guild_id = msg.guild_id.unwrap();

        let player = state.lavalink.player(guild_id).await.unwrap();
        let (parts, body) = twilight_lavalink::http::load_track(
            player.node().config().address,
            &msg_split[1],
            &player.node().config().authorization,
        )?
        .into_parts();
        let req = Request::from_parts(parts, Body::from(body));
        let res = state.hyper.request(req).await?;
        let response_bytes = hyper::body::to_bytes(res.into_body()).await?;

        let loaded = serde_json::from_slice::<LoadedTracks>(&response_bytes)?;

        if let Some(track) = loaded.tracks.first() {
            player.send(Play::from((guild_id, &track.track)))?;

            let content = format!(
                "Playing **{:?}** by **{:?}**",
                track.info.title, track.info.author
            );
            state
                .http
                .create_message(msg.channel_id)
                .content(content)?
                .await?;
        } else {
            state
                .http
                .create_message(msg.channel_id)
                .content("Didn't find any results")?
                .await?;
        }
    } else {
        state
            .http
            .create_message(msg.channel_id)
            .content("Too many args!")?
            .await?;
    }

    Ok(())
}

async fn pause(msg: Message, state: State) -> Result<(), Box<dyn Error + Send + Sync + 'static>> {
    debug!(
        "pause command in channel {} by {}",
        msg.channel_id, msg.author.name
    );

    let guild_id = msg.guild_id.unwrap();
    let player = state.lavalink.player(guild_id).await.unwrap();
    let paused = player.paused();
    player.send(Pause::from((guild_id, !paused)))?;

    let action = if paused { "Unpaused " } else { "Paused" };

    state
        .http
        .create_message(msg.channel_id)
        .content(format!("{} the track", action))?
        .await?;

    Ok(())
}

async fn seek(msg: Message, state: State) -> Result<(), Box<dyn Error + Send + Sync + 'static>> {
    debug!(
        "seek command in channel {} by {}",
        msg.channel_id, msg.author.name
    );
    state
        .http
        .create_message(msg.channel_id)
        .content("Where in the track do you want to seek to (in seconds)?")?
        .await?;

    let author_id = msg.author.id;
    let msg = state
        .standby
        .wait_for_message(msg.channel_id, move |new_msg: &MessageCreate| {
            new_msg.author.id == author_id
        })
        .await?;
    let guild_id = msg.guild_id.unwrap();
    let position = msg.content.parse::<i64>()?;

    let player = state.lavalink.player(guild_id).await.unwrap();
    player.send(Seek::from((guild_id, position * 1000)))?;

    state
        .http
        .create_message(msg.channel_id)
        .content(format!("Seeked to {}s", position))?
        .await?;

    Ok(())
}

async fn stop(msg: Message, state: State) -> Result<(), Box<dyn Error + Send + Sync + 'static>> {
    debug!(
        "stop command in channel {} by {}",
        msg.channel_id, msg.author.name
    );

    let guild_id = msg.guild_id.unwrap();
    let player = state.lavalink.player(guild_id).await.unwrap();
    player.send(Stop::from(guild_id))?;

    state
        .http
        .create_message(msg.channel_id)
        .content("Stopped the track")?
        .await?;

    Ok(())
}

async fn volume(msg: Message, state: State) -> Result<(), Box<dyn Error + Send + Sync + 'static>> {
    debug!(
        "volume command in channel {} by {}",
        msg.channel_id, msg.author.name
    );
    state
        .http
        .create_message(msg.channel_id)
        .content("What's the volume you want to set (0-1000, 100 being the default)?")?
        .await?;

    let author_id = msg.author.id;
    let msg = state
        .standby
        .wait_for_message(msg.channel_id, move |new_msg: &MessageCreate| {
            new_msg.author.id == author_id
        })
        .await?;
    let guild_id = msg.guild_id.unwrap();
    let volume = msg.content.parse::<i64>()?;

    if !(0..=1000).contains(&volume) {
        state
            .http
            .create_message(msg.channel_id)
            .content("That's more than 1000")?
            .await?;

        return Ok(());
    }

    let player = state.lavalink.player(guild_id).await.unwrap();
    player.send(Volume::from((guild_id, volume)))?;

    state
        .http
        .create_message(msg.channel_id)
        .content(format!("Set the volume to {}", volume))?
        .await?;

    Ok(())
}
