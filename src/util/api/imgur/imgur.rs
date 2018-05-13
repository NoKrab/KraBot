use CONFIG;
use util::network::request::request::SimpleRequest;
use std::sync::Mutex;
use hyper::header::{ContentType, Headers, Authorization, Bearer};
use hyper::{Method};
use postgres::rows::{Rows, Row};
use serde_json::Value;
use database::postgres::postgres as pg_backend;

lazy_static! {
    static ref ACCESS_TOKEN: Mutex<String> = Mutex::new("".to_string());
    static ref REFRESH_TOKEN: Mutex<String> = {
        if let Some(ref refresh_token) = CONFIG.optional.imgur_refresh_token {
            return Mutex::new(refresh_token.to_owned());
        } else {
            return Mutex::new("".to_string());
        }
    };
    static ref CLIENT_ID: String = {
        if let Some(ref client_id) = CONFIG.optional.imgur_client_id {
            return client_id.to_owned();
        } else {
            return "".to_string();
        }
    };
   static ref CLIENT_SECRET: String = {
        if let Some(ref client_secret) = CONFIG.optional.imgur_client_secret {
            return client_secret.to_owned();
        } else {
            return "".to_string();
        };
    };
}


pub struct Account {}
pub struct Comment {}
pub struct Album {}
pub struct Gallery {}
pub struct Image {}

impl Account {
    pub fn generate_access_token() {
        let mut headers = Headers::new();
        headers.set(ContentType::form_url_encoded());

        let data = format!(
            "refresh_token={}&&client_id={}&&client_secret={}&&grant_type=refresh_token",
            REFRESH_TOKEN.lock().unwrap().to_string(),
            CLIENT_ID.to_string(),
            CLIENT_SECRET.to_string()
        );
        if let Some(result) = SimpleRequest::new().headers(headers).uri("https://api.imgur.com/oauth2/token".to_string()).body(data).method(Method::Post).run() {
            let mut access_token = ACCESS_TOKEN.lock().unwrap();
            let mut refresh_token = REFRESH_TOKEN.lock().unwrap();

            access_token.clear();
            access_token.insert_str(0, &result["access_token"].as_str().unwrap());
            refresh_token.clear();
            refresh_token.insert_str(0, &result["refresh_token"].as_str().unwrap());

            info!("generate_access_token -> {}", access_token);
            info!("generate_refresh_token -> {}", refresh_token);
        }
    }

    pub fn account_images() -> Option<Value> {
        let mut headers = Headers::new();
        headers.set(
            Authorization(
                Bearer {
                    token: ACCESS_TOKEN.lock().unwrap().to_owned()
                }
            )
        );
        let request = SimpleRequest::new().headers(headers).uri("https://api.imgur.com/3/account/me/images".to_string()).method(Method::Get);
        if let Some(result) = make_imgur_request(request) {
            if result["success"] == true {
                return Some(result["data"].to_owned());
            } else {
                return None;
            }
        } else {
            return None;
        }
    }

    pub fn albums() -> Option<Value> {
        let mut headers = Headers::new();
        headers.set(
            Authorization(
                Bearer {
                    token: ACCESS_TOKEN.lock().unwrap().to_owned()
                }
            )
        );
        let request = SimpleRequest::new().headers(headers).uri("https://api.imgur.com/3/account/me/albums".to_string()).method(Method::Get);
        if let Some(result) = make_imgur_request(request) {
            if result["success"] == true {
                return Some(result["data"].to_owned());
            } else {
                return None;
            }
        } else {
            return None;
        }
    }
}

impl Album {
    pub fn album_images(guild_id: i64) -> Option<Value> {
        let mut headers = Headers::new();
        headers.set(
            Authorization(
                Bearer {
                    token: ACCESS_TOKEN.lock().unwrap().to_owned()
                }
            )
        );
        let uri = format!(
            "https://api.imgur.com/3/album/{}/images",
            get_current_album_id(guild_id)

        );
        let request = SimpleRequest::new().headers(headers).uri(uri).method(Method::Get);
        if let Some(result) = make_imgur_request(request) {
            if result["success"] == true {
                return Some(result["data"].to_owned());
            } else {
                return None;
            }
        } else {
            return None;
        }
    }
}

fn make_imgur_request(request: SimpleRequest) -> Option<Value> {
    if let Some(result) = request.clone().run() {
        if result["status"] == 403 {
            info!("Requesting new token...");
            Account::generate_access_token();
            return request.clone().headers({
                let mut headers = request.get_headers().unwrap();
                headers.set(
                    Authorization(
                        Bearer {
                            token: ACCESS_TOKEN.lock().unwrap().to_owned()
                        }
                    )
                );
                headers
            }).run();
        }
        return Some(result);
    } else {
       return None;
    }
}

pub fn set_album_id(album_id: &str, guild_id: i64) {
    pg_backend::execute_sql("UPDATE settings SET imgur_album_id = $1 WHERE guild_id = $2", &[&album_id, &guild_id]);
}

pub fn get_current_album_id(guild_id: i64) -> String {
    if let Some(rows) = pg_backend::query_sql("SELECT imgur_album_id FROM settings WHERE guild_id = $1", &[&guild_id]) {
        let mut album_id: String = "".to_string();
        for row in &rows {
            let album_id_row: String = row.get(0);
            debug!("Current Album: {}", album_id_row);
            album_id = album_id_row;
        }
        return album_id;
    } else {
        return "No album set.".to_string();
    }
}