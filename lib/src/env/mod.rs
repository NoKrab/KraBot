use std::env;

pub fn print_all_env() {
    for (key, value) in env::vars() {
        debug!("{} : {}", key, value);
    }
}

pub fn get_discord_token() -> String {
    let key = "DISCORD_TOKEN";
    match env::var(key) {
        Ok(val) => val,
        Err(e) => panic!("couldn't interpret {}: {}", key, e),
    }
}

pub fn get_lavalink_env() -> (String, u16, String) {
    let host = "LAVALINK_HOST";
    let port = "LAVALINK_PORT";
    let auth = "LAVALINK_AUTHORIZATION";
    let host = match env::var(host) {
        Ok(val) => val,
        Err(e) => panic!("couldn't interpret {}: {}", host, e),
    };
    let port = match env::var(port) {
        Ok(val) => val.parse::<u16>().unwrap(),
        Err(e) => panic!("couldn't interpret {}: {}", port, e),
    };
    let auth = match env::var(auth) {
        Ok(val) => val,
        Err(e) => panic!("couldn't interpret {}: {}", auth, e),
    };
    (host, port, auth)
}

pub fn get_bot_prefix() -> String {
    let key = "RSBOT_PREFIX";
    match env::var(key) {
        Ok(val) => val,
        Err(e) => panic!("couldn't interpret {}: {}", key, e),
    }
}
