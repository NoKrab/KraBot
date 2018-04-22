use CONFIG;
use util::network::request::request::SimpleRequest;
//use serenity::prelude::Mutex;
use std::sync::Mutex;

lazy_static! {
    static ref ACCESS_TOKEN: Mutex<String> = Mutex::new("".to_string());
    static ref REFRESH_TOKEN: String = {
        if let Some(ref refresh_token) = CONFIG.optional.imgur_refresh_token {
            return refresh_token.to_owned();
        } else {
            return "".to_string();
        }
    };
}


pub struct Account {}
pub struct Comment {}
pub struct Album {}
pub struct Gallery {}
pub struct Image {}

impl Account {
    fn generate_access_token() {
        let mut access_token = ACCESS_TOKEN.lock().unwrap();
        access_token.clear();
        access_token.insert_str(0, "new token");
        info!("{}", access_token);
    }
}