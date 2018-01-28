use toml;
use std::fs::File;
use std::io::{BufReader, Read};

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
    pub sqlite_path: String,
}

#[derive(Debug, Deserialize)]
pub struct Optional {
    pub database_name: Option<String>,
    pub setting: Option<bool>,
}

impl Config {
    fn read_config(path: &str) -> Config {
        let mut config = String::new();

        let f = File::open(path).expect("Unable to open file");
        let mut br = BufReader::new(f);
        br.read_to_string(&mut config)
            .expect("Unable to read string");
        println!("{}", config);
        let mut config: Config = toml::from_str(&config).unwrap();
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
    pub fn get_sqlite_path(path: &str) -> String {
        let config = Config::read_config(path);
        let mut path = config.required.sqlite_path;
        let db_name = config.optional.database_name.as_ref().unwrap();
        let trailing_char = path.chars().nth(path.len() - 1).unwrap();
        match trailing_char {
            '/' => (),
            _ => path.push_str("/"),
        }
        let path = path + db_name;
        path
    }
}
