use serenity::http;

command!(quit(ctx, msg, _args) {
    ctx.quit();

    let _ = msg.reply("Shutting down!");
});

command!(clear(_ctx, msg, _args) {
    let channel_id = msg.channel_id.0;
    debug!("ChannelID: {:#?}", channel_id);
//    let channel_num = channel_id as u64;
    let _messages = match  http::get_messages(channel_id, &"/search?content=clear") {
        Ok(msgs) =>  {
            info!("Messages: {:#?}", msgs);
            msgs
        },
        Err(_) => {
            error!("No messages found!");
            return Ok(());
        }
    };

});
