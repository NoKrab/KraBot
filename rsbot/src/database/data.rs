use serenity::client::CACHE;

// i32 will crash
pub fn get_guild_ids() -> Vec<u64> {
    let mut guild_ids: Vec<u64> = Vec::new();

    let cache = &CACHE.read().guilds;
    for (x, _) in cache.iter() {
        // This cannot be the final solution
        guild_ids.push(x.to_string().parse::<u64>().unwrap());
    }
    guild_ids
}
