use serenity::http;

command!(quit(ctx, msg, _args) {
    ctx.quit();

    let _ = msg.reply("Shutting down!");
});

command!(clear(ctx, msg, _args) {
    let channel_id = msg.channel_id.0;
    println!("ChannelID: {:#?}", channel_id);
//    let channel_num = channel_id as u64;
    let messages = match  http::get_messages(channel_id, &"/search?content=clear") {
        Ok(msgs) =>  {
            println!("Messages: {:#?}", msgs);
            msgs
        },
        Err(_) => {
            println!("No messages found!");
            return Ok(());
        }
    };

});