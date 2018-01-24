command!(ping(_ctx, msg) {
    let _ = msg.channel_id.say("Pong!");
});

command!(uptime(_ctx, msg) {
    let _ = msg.channel_id.say("Uptime!");
});