use std::fmt::Write;
use CommandCounter;
use DIESEL_PG;
use chrono::Utc;

command!(ping(_ctx, msg) {
    let _ = msg.channel_id.say("Pong!");
    info!("Shard {}", _ctx.shard_id);
});

command!(uptime(ctx, msg) {
    let shard = DIESEL_PG.get_shard_timestamp(ctx.shard_id as i32).unwrap();
    let duration = Utc::now().naive_utc().signed_duration_since(shard.chrono_timestamp);

    let mut secs_total = duration.num_seconds();
    let days = (secs_total / (86400)) as u32;
    secs_total %= 86400;
    let hours = (secs_total / (60 * 60)) as u32;
    secs_total %= 3600;
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
