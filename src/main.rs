#[macro_use]
extern crate serde_derive;
#[macro_use]
extern crate serenity;
extern crate toml;
extern crate typemap;

mod config;

use std::collections::HashMap;
use std::fmt::Write;
use std::sync::Arc;
use config::Config;
use typemap::Key;
use serenity::client::bridge::gateway::{ShardId, ShardManager};
use serenity::framework::standard::{help_commands, Args, CommandOptions, DispatchError,
                                    HelpBehaviour, StandardFramework};
use serenity::model::channel::Message;
use serenity::model::gateway::Ready;
use serenity::model::Permissions;
use serenity::prelude::Mutex;
use serenity::prelude::*;

struct ShardManagerContainer;

impl Key for ShardManagerContainer {
    type Value = Arc<Mutex<ShardManager>>;
}

struct CommandCounter;

impl Key for CommandCounter {
    type Value = HashMap<String, u64>;
}

struct Handler;

impl EventHandler for Handler {
    fn ready(&self, _: Context, ready: Ready) {
        println!("{} is connected!", ready.user.name);
    }
}

fn main() {
    let config = Config::read_config("./config/config.toml");
    Config::hello();
    println!("{:?}", config);
    /*assert_eq!(config.required.token, "ibimseintoken");
    assert_eq!(config.optional.setting, true);*/
    assert_eq!(config.required.prefix, "!");

    let mut client = Client::new(&config.required.token, Handler).expect("Error creating client");

    {
        let mut data = client.data.lock();
        data.insert::<CommandCounter>(HashMap::default());
        data.insert::<ShardManagerContainer>(Arc::clone(&client.shard_manager));
    }

    client.with_framework(
        StandardFramework::new()
            .configure(|c| {
                c.allow_whitespace(true)
                    .on_mention(true)
                    .prefix(&config.required.prefix)
                    .delimiters(vec![", ", ","])
            })
            .before(|ctx, msg, command_name| {
                println!(
                    "Got command '{}' by user '{}'",
                    command_name, msg.author.name
                );
                let mut data = ctx.data.lock();
                let counter = data.get_mut::<CommandCounter>().unwrap();
                let entry = counter.entry(command_name.to_string()).or_insert(0);
                *entry += 1;

                true
            })
            .after(|_, _, command_name, error| match error {
                Ok(()) => println!("Processed command '{}'", command_name),
                Err(why) => println!("Command '{}' returned error {:?}", command_name, why),
            })
            .on_dispatch_error(|_ctx, msg, error| {
                if let DispatchError::RateLimited(seconds) = error {
                    let _ = msg.channel_id
                        .say(&format!("Try this again in {} seconds", seconds));
                }
            })
            .simple_bucket("emoji", 5)
            .bucket("complicated", 5, 30, 2)
            .command("about", |c| c.cmd(about))
            .customised_help(help_commands::with_embeds, |c| {
                c.individual_command_tip("Hello! こんにちは！Hola! Bonjour! 您好!\n\
                If you want more information about a specific command, just pass the command as argument.")
                .command_not_found_text("Could not {}, I'm sorry : (")
                .suggestion_text("How about this command: {}, it's numero uno on the market...!")
                .lacking_permissions(HelpBehaviour::Hide)
                .lacking_role(HelpBehaviour::Nothing)
                .wrong_channel(HelpBehaviour::Strike)
            })
            .command("commands", |c| c.bucket("complicated").cmd(commands))
            .group("Emoji", |g| {
                g.prefix("emoji")
                    .command("cat", |c| {
                        c.desc("Sends an emoji with a cat.")
                            .batch_known_as(vec!["kitty", "neko"])
                            .bucket("emoji")
                            .cmd(cat)
                            .required_permissions(Permissions::ADMINISTRATOR)
                    })
                    .command("dog", |c| {
                        c.desc("Sends an emoji with a dog.")
                            .bucket("emoji")
                            .cmd(dog)
                    })
            })
            .command("multiply", |c| c.known_as("*").cmd(multiply))
            .command("latency", |c| c.cmd(latency))
            .command("ping", |c| c.check(owner_check).cmd(ping))
            .command("role", |c| {
                c.cmd(about_role)
                    .allowed_roles(vec!["mods", "ultimate neko"])
            })
            .command("some long command", |c| c.cmd(some_long_command)),
    );

    if let Err(why) = client.start() {
        println!("Client error: {:?}", why);
    }

    /*if let Err(why) = client.start_shards(2) {
        println!("Client error {:?}", why);
    }*/
}

command!(commands(ctx, msg, _args) {
    let mut contents = "Commands used:\n".to_string();

    let data = ctx.data.lock();
    let counter = data.get::<CommandCounter>().unwrap();

    for (k, v) in counter {
        let _ = write!(contents, "- {name}: {amount}\n", name=k, amount=v);
    }

    if let Err(why) = msg.channel_id.say(&contents) {
        println!("Error sending message: {:?}", why);
    }
});

// A function which acts as a "check", to determine whether to call a command.
//
// In this case, this command checks to ensure you are the owner of the message
// in order for the command to be executed. If the check fails, the command is
// not called.
fn owner_check(_: &mut Context, msg: &Message, _: &mut Args, _: &CommandOptions) -> bool {
    // Replace 7 with your ID
    msg.author.id == 217447645856268292
}

command!(some_long_command(_ctx, msg, args) {
    if let Err(why) = msg.channel_id.say(&format!("Arguments: {:?}", args)) {
        println!("Error sending message: {:?}", why);
    }
});

command!(about_role(_ctx, msg, args) {
    let potential_role_name = args.full();

    if let Some(guild) = msg.guild() {
        // `role_by_name()` allows us to attempt attaining a reference to a role
        // via its name.
        if let Some(role) = guild.read().role_by_name(&potential_role_name) {
            if let Err(why) = msg.channel_id.say(&format!("Role-ID: {}", role.id)) {
                println!("Error sending message: {:?}", why);
            }

            return Ok(());
        }
    }

    if let Err(why) = msg.channel_id.say(
                      &format!("Could not find role named: {:?}", potential_role_name)) {
        println!("Error sending message: {:?}", why);
    }
});

// Using the `command!` macro, commands can be created with a certain type of
// "dynamic" type checking. This is a method of requiring that the arguments
// given match the required type, and maps those arguments to the specified
// bindings.
//
// For example, the following will be correctly parsed by the macro:
//
// `~multiply 3.7 4.3`
//
// However, the following will not, as the second argument can not be an f64:
//
// `~multiply 3.7 four`
//
// Since the argument can't be converted, the command returns early.
//
// Additionally, if not enough arguments are given (e.g. `~multiply 3`), then
// the command will return early. If additional arguments are provided, they
// will be ignored.
//
// Argument type overloading is currently not supported.
command!(multiply(_ctx, msg, args) {
    let first = args.single::<f64>().unwrap();
    let second = args.single::<f64>().unwrap();

    let res = first * second;

    if let Err(why) = msg.channel_id.say(&res.to_string()) {
        println!("Err sending product of {} and {}: {:?}", first, second, why);
    }
});

command!(about(_ctx, msg, _args) {
    if let Err(why) = msg.channel_id.say("This is a small test-bot! : )") {
        println!("Error sending message: {:?}", why);
    }
});

command!(latency(ctx, msg, _args) {
    // The shard manager is an interface for mutating, stopping, restarting, and
    // retrieving information about shards.
    let data = ctx.data.lock();

    let shard_manager = match data.get::<ShardManagerContainer>() {
        Some(v) => v,
        None => {
            let _ = msg.reply("There was a problem getting the shard manager");

            return Ok(());
        },
    };

    let manager = shard_manager.lock();
    let runners = manager.runners.lock();

    // Shards are backed by a "shard runner" responsible for processing events
    // over the shard, so we'll get the information about the shard runner for
    // the shard this command was sent over.
    let runner = match runners.get(&ShardId(ctx.shard_id)) {
        Some(runner) => runner,
        None => {
            let _ = msg.reply("No shard found");

            return Ok(());
        },
    };

    let _ = msg.reply(&format!("The shard latency is {:?}", runner.latency));
});

command!(ping(_ctx, msg, _args) {
    if let Err(why) = msg.channel_id.say("Pong! : )") {
        println!("Error sending message: {:?}", why);
    }
});

command!(dog(_ctx, msg, _args) {
    if let Err(why) = msg.channel_id.say(":dog:") {
        println!("Error sending message: {:?}", why);
    }
});

command!(cat(_ctx, msg, _args) {
    if let Err(why) = msg.channel_id.say(":cat:") {
        println!("Error sending message: {:?}", why);
    }
});
