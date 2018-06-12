use database::sqlite::sqlite;
use std::fmt::Write;
use CommandCounter;
use SQLITE_PATH;

command!(ping(_ctx, msg) {
    let _ = msg.channel_id.say("Pong!");
    info!("Shard {}", _ctx.shard_id);
});

command!(uptime(_ctx, msg) {
    let con = sqlite::create_connection(&*SQLITE_PATH);
    let stm = sqlite::select_shard_uptime(&con, _ctx.shard_id as i64).unwrap();
    let _ = con.close().expect("Failed to close connection");

    let secs_total = stm.num_seconds();
    let days = (secs_total / (60 * 60 * 24)) as u32;
    let hours = (secs_total / (60 * 60)) as u32;
    let minutes = (secs_total / 60) as u32;
    let secounds = (secs_total % 60) as u32;

    let _ = msg.channel_id.say(format!("{}d {}h {}m {}s", 
        days,
        hours,
        minutes,
        secounds
    ));
});

command!(commands(ctx, msg, _args) {
    let mut contents = "Commands used:\n".to_string();
    let data = ctx.data.lock();
    let counter = data.get::<CommandCounter>().unwrap();

    for (k, v) in counter {
        let _ = write!(contents, "- {name}: {amount}\n", name=k, amount=v);
    }

    if let Err(why) = msg.channel_id.say(&contents) {
        error!("Error sending message: {:?}", why);
    }
});
