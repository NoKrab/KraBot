use std::collections::HashMap;
use std::collections::HashSet;
use chrono::prelude::*;
use serenity::framework::standard::{help_commands, DispatchError, HelpBehaviour, StandardFramework};
use serenity::http;
use typemap::Key;
use CONFIG;

use commands;

pub struct CommandCounter;

impl Key for CommandCounter {
    type Value = HashMap<String, u64>;
}

pub fn get_framework() -> StandardFramework {
    let owners = match http::get_current_application_info() {
        Ok(info) => {
            let mut set = HashSet::new();
            set.insert(info.owner.id);

            set
        }
        Err(why) => {
            error!("Couldn't get application info: {}", why);
            panic!()
        }
    };
    let framework: StandardFramework = {
        let mut f: StandardFramework = StandardFramework::new()
            .configure(|c| c.owners(owners).prefix(&*CONFIG.required.prefix).on_mention(CONFIG.required.mention).delimiters(vec![", ", ","]))
            .before(|ctx, msg, cmd_name| {
                debug!("Got command '{}' by user '{}'", cmd_name, msg.author.name);
                let mut data = ctx.data.lock();
                let counter = data.get_mut::<CommandCounter>().unwrap();
                let entry = counter.entry(cmd_name.to_string()).or_insert(0);
                *entry += 1;
                true
            })
            .after(|_, msg, cmd_name, error| match error {
                Ok(()) => debug!("Command '{}' completed in {:?}", cmd_name, Utc::now().signed_duration_since(msg.timestamp)),
                Err(why) => error!("Command '{}' returned error {:?}", cmd_name, why),
            })
            .unrecognised_command(|_, _, unknown_command_name| {
                debug!("Could not find command named '{}'", unknown_command_name);
            })
            .on_dispatch_error(|_ctx, msg, error| {
                if let DispatchError::RateLimited(seconds) = error {
                    let _ = msg.channel_id.say(&format!("Try this again in {} seconds", seconds));
                }
            })
            .customised_help(help_commands::with_embeds, |c| {
                // This replaces the information that a user can pass
                // a command-name as argument to gain specific information about it.
                c.individual_command_tip(
                    "Hello! こんにちは！Hola! Bonjour! 您好!\n\
                     If you want more information about a specific command, just pass the command as argument.",
                )
                // Some arguments require a `{}` in order to replace it with contextual information.
                // In this case our `{}` refers to a command's name.
                .command_not_found_text("Could not {}, I'm sorry : (")
                // Another argument requiring `{}`, again replaced with the command's name.
                .suggestion_text("How about this command: {}, it's numero uno on the market...!")
                // On another note, you can set up the help-menu-filter-behaviour.
                // Here are all possible settings shown on all possible options.
                // First case is if a user lacks permissions for a command, we can hide the command.
                .lacking_permissions(HelpBehaviour::Hide)
                // If the user is nothing but lacking a certain role, we just display it hence our variant is `Nothing`.
                .lacking_role(HelpBehaviour::Nothing)
                // The last `enum`-variant is `Strike`, which ~~strikes~~ a command.
                .wrong_channel(HelpBehaviour::Strike)
                // Serenity will automatically analyse and generate a hint/tip explaining the possible
                // cases of a command being ~~striked~~, but only  if
                // `striked_commands_tip(Some(""))` keeps `Some()` wrapping an empty `String`, which is the default value.
                // If the `String` is not empty, your given `String` will be used instead.
                // If you pass in a `None`, no hint will be displayed at all.
            })
            .command("ping", |c| c.cmd(commands::meta::ping))
            .command("uptime", |c| c.cmd(commands::meta::uptime))
            .command("quit", |c| c.cmd(commands::owner::quit).owners_only(true))
            .command("clear", |c| c.cmd(commands::owner::clear).owners_only(true))
            .command("host", |c| c.cmd(commands::owner::host).owners_only(true))
            .command("save", |c| c.cmd(commands::owner::save).owners_only(true))
            .command("load", |c| c.cmd(commands::owner::load).owners_only(true))
            .group("Fun", |g| g.command("t", |c| c.cmd(commands::fun::twitch)).command("flip", |c| c.cmd(commands::fun::flip)))
            .group("Math", |g| {
                let mut g = g
                    .command("join", |c| c.cmd(commands::voice::join))
                    .command("multiply", |c| c.cmd(commands::math::multiply))
                    .command("fib", |c| c.cmd(commands::math::fibonacci))
                    .command("calc", |c| {
                        c.desc(
                            "Tries to calculate the given expressions. \nSupported operators are:\n\
                             ! \n!=\n \n\"\" \n'' \n() \n[] \n, \n>, <, >=, <=, == \n+, -, *, /, %, \n&&, ||\n\
                             Built-in functions: \n\nmin(), max(), len(), is_empty(), array()",
                        )
                        .cmd(commands::math::calc)
                    });
                g
            })
            .group("Voice", |g| {
                let mut g = g
                    .command("join", |c| c.cmd(commands::voice::join))
                    .command("leave", |c| c.cmd(commands::voice::leave))
                    .command("play", |c| c.cmd(commands::voice::play))
                    .command("mute", |c| c.cmd(commands::voice::mute))
                    .command("unmute", |c| c.cmd(commands::voice::unmute))
                    .command("deafen", |c| c.cmd(commands::voice::deafen))
                    .command("undeafen", |c| c.cmd(commands::voice::undeafen))
                    .command("stop", |c| c.cmd(commands::voice::stop));
                if let Some(ref youtube_token) = CONFIG.optional.youtube_token {
                    info!("Youtube API enabled.");
                    g = g.command("search", |c| c.cmd(commands::voice::search));
                }
                g
            });
        if let Some(ref imgur_client_id) = CONFIG.optional.imgur_client_id {
            info!("Imgur API enabled.");
            f = f.group("Imgur", |g| {
                // let mut g = g.command("imgs", |c| c.cmd(commands::imgur::get_imgs))
                let mut g = g
                    .command("albums", |c| c.cmd(commands::imgur::get_albums))
                    .command("set_album", |c| c.cmd(commands::imgur::set_album))
                    .command("get_current_album", |c| c.cmd(commands::imgur::get_current_album))
                    .command("img", |c| c.cmd(commands::imgur::query_img));
                g
            });
        }

        f = f.command("commands", |c| c.cmd(commands::meta::commands));
        f
    };
    framework
}
