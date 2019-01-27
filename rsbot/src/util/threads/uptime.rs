use serenity::model::gateway::Game;
use serenity::prelude::Context;
use std::thread;
use std::time::Duration;
use DIESEL_PG;
use chrono::Utc;

pub fn init(ctx: Context) {
    info!("Spawned uptime thread");
    thread::spawn(move || loop {
        let shard = DIESEL_PG.get_shard_timestamp(ctx.shard_id as i32).unwrap();
        let duration = Utc::now().naive_utc().signed_duration_since(shard.chrono_timestamp);

        let mut secs_total = duration.num_seconds();
        let days = (secs_total / (86400)) as u32;
        secs_total %= 86400;
        let hours = (secs_total / (60 * 60)) as u32;
        secs_total %= 3600;
        let minutes = (secs_total / 60) as u32;
        let seconds = (secs_total % 60) as u32;

        ctx.set_game(Game::playing(&format!("{}d {}h {}m {}s", days, hours, minutes, seconds)));

        thread::sleep(Duration::from_secs(60));
    });
}
