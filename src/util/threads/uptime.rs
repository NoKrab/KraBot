use database::sqlite::sqlite;
use serenity::model::gateway::Game;
use serenity::prelude::Context;
use std::thread;
use std::time::Duration;
use SQLITE_PATH;

pub fn init(ctx: Context) {
    info!("Spawned uptime thread");
    thread::spawn(move || loop {
        let con = sqlite::create_connection(&*SQLITE_PATH);
        let stm = sqlite::select_shard_uptime(&con, ctx.shard_id as i64).unwrap();
        let _ = con.close().expect("Failed to close connection");

        let mut secs_total = stm.num_seconds();
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
