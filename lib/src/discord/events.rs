use lavalink_rs::{
    async_trait,
    gateway::LavalinkEventHandler,
    model::{TrackFinish, TrackStart},
    LavalinkClient,
};
use serenity::{
    client::{Context, EventHandler},
    model::{id::GuildId, prelude::Ready},
};

pub(crate) struct Handler;
pub(crate) struct LavalinkHandler;

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
    async fn player_update(
        &self,
        _client: LavalinkClient,
        _event: lavalink_rs::model::PlayerUpdate,
    ) {
        debug!("{:#?}", _event);
    }
}
