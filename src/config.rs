use toml;
use std::fs::File;
use std::io::{Read, BufReader};

#[derive(Debug, Deserialize)]
pub struct Config {
    pub required: Required,
    pub optional: Optional,
}

#[derive(Debug, Deserialize)]
pub struct Required {
    pub token: String,
    pub prefix: String,
}

#[derive(Debug, Deserialize)]
pub struct Optional {
    pub setting: bool,
}

impl Config {
    pub fn read_config(path: &str) -> Config {
        let mut config = String::new();
        let f = File::open(path)
            .expect("Unable to open file");
        let mut br = BufReader::new(f);
        br.read_to_string(&mut config)
            .expect("Unable to read string");
        let config: Config = toml::from_str(&config).unwrap();
        config
    }
}