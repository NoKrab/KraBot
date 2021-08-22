use serenity::{
    client::Context,
    framework::standard::{macros::hook, CommandResult},
    model::channel::Message,
};

#[hook]
pub(crate) async fn after(
    ctx: &Context,
    msg: &Message,
    command_name: &str,
    command_result: CommandResult,
) {
    if let Err(e) = command_result {
        error!("Command '{}' returned error {:?} => {}", command_name, e, e);
        let _ = msg.react(ctx, '❌').await;
    } else {
        let _ = msg.react(ctx, '✅').await;
    }
}
