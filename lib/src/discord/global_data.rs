use std::{sync::Arc, time::Instant};

use lavalink_rs::LavalinkClient;
use serenity::prelude::TypeMapKey;

pub(crate) struct Uptime;
pub(crate) struct Lavalink;

impl TypeMapKey for Lavalink {
    type Value = LavalinkClient;
}

impl TypeMapKey for Uptime {
    type Value = Arc<Instant>;
}
