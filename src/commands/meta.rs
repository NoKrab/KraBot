use SQLITE_PATH;
use database::sqlite::sqlite;
use CommandCounter;
use std::fmt::Write;

command!(ping(_ctx, msg) {
    let _ = msg.channel_id.say("Pong!");
    info!("Shard {}", _ctx.shard_id);
});

command!(uptime(_ctx, msg) {
    let con = sqlite::create_connection(&*SQLITE_PATH);
    let stm = sqlite::select_shard_uptime(&con, _ctx.shard_id as i64).unwrap();
    let _ = con.close().expect("Failed to close connection");
    debug!("{}", stm);
    let _ = msg.channel_id.say(&format!("Uptime! Initial Connection time: {:#?}", stm));
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
