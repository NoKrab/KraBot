use serenity::client::validate_token;
use std::fs;
use std::fs::File;
use std::io::{BufReader, Read};
use std::path::Path;
use toml;

pub const CONFIG_PATH: &str = "./config/config.toml";

#[derive(Debug, Deserialize)]
pub struct Config {
    pub required: Required,
    pub optional: Optional,
}

#[derive(Debug, Deserialize)]
pub struct Required {
    pub token: String,
    pub prefix: String,
    pub mention: bool,
    pub shards: u64,
    pub sqlite_path: String,
    pub pg_connection_url: String,
}

#[derive(Debug, Deserialize)]
pub struct Optional {
    pub database_name: Option<String>,
    pub youtube_token: Option<String>,
    pub setting: Option<bool>,
    pub imgur_client_id: Option<String>,
    pub imgur_refresh_token: Option<String>
}

impl Config {
    fn read_config(path: &str) -> Config {
        let mut config = String::new();

        // TODO catch open file errors
        if !Path::new(&CONFIG_PATH).exists() {
            fs::copy("./config/config_example.toml", &CONFIG_PATH).expect("Error copying file");
            println!("I created the config.toml for you, please be sure to insert your token accordingly!");
            ::std::process::exit(0);
        }
        let f = File::open(path).expect("Unable to open config.toml file");
        let mut br = BufReader::new(f);
        br.read_to_string(&mut config).expect("Unable to read string");
        // println!("{}", config);
        // let mut config: Config = toml::from_str(&config).unwrap();
        let mut config: Config = match toml::from_str(&config) {
            Ok(config) => config,
            Err(why) => panic!("Something bad has happened! Check your config.toml. Error Message: {:?}", why),
        };
        match validate_token(&config.required.token) {
            Err(_) => panic!("Token is invalid"),
            Ok(()) => (),
        };
        if config.optional.database_name == None {
            config = Config::default_db(config);
        } else if config.optional.database_name.as_ref().unwrap().is_empty() {
            config = Config::default_db(config);
        }
        config
    }
    fn default_db(mut config: Config) -> Config {
        let default_name: Option<String> = Some("rsbot.db".to_string());
        config.optional.database_name = default_name;
        config
    }
    pub fn get_config(path: &str) -> Config {
        let config = Config::read_config(path);
        config
    }
    pub fn get_sqlite_path(path: &str) -> (String, String) {
        let config = Config::read_config(path);
        let mut path = config.required.sqlite_path;
        let clone = path.clone();
        let db_name = config.optional.database_name.unwrap_or(String::from("rsbot.db"));
        let trailing_char = path.chars().nth(path.len() - 1).unwrap();
        match trailing_char {
            '/' => (),
            _ => path.push_str("/"),
        }
        let path = path + &db_name;
        (path, clone)
    }
}
