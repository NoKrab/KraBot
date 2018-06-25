use serenity::prelude::Context;
use database::sqlite::sqlite;
use SQLITE_PATH;
use std::thread;
use std::time::Duration;



pub fn init(ctx: Context) {
    debug!("Spawned uptime thread");
    thread::spawn(move || loop {
        thread::sleep(Duration::from_secs(60));
        let con = sqlite::create_connection(&*SQLITE_PATH);
        let stm = sqlite::select_shard_uptime(&con, ctx.shard_id as i64).unwrap();
        let _ = con.close().expect("Failed to close connection");

        let secs_total = stm.num_seconds();
        let days = (secs_total / (60 * 60 * 24)) as u32;
        let hours = (secs_total / (60 * 60)) as u32;
        let minutes = (secs_total / 60) as u32;
        let secounds = (secs_total % 60) as u32;


        ctx.set_game_name(&format!("{}d {}h {}m {}s", 
            days,
            hours,
            minutes,
            secounds
        ));
  
    });
}