command!(ping(_ctx, msg) {
    let _ = msg.channel_id.say("Pong!");
    println!("Shard {}", _ctx.shard_id);
});

command!(uptime(_ctx, msg) {
    let _ = msg.channel_id.say("Uptime!");
//    println!("{:?}", _ctx.data);
});