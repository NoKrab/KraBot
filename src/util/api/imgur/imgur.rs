use super::models::AccessToken;
use database::postgres::postgres as pg_backend;
use hyper::header::{Authorization, Bearer, ContentType, Headers};
use hyper::Method;
use postgres::rows::{Row, Rows};
use reqwest::header::{HeaderMap, HeaderValue, CONTENT_TYPE, USER_AGENT};
use serde_json::Value;
use std::error::Error;
use std::sync::Mutex;
use util::network::request::request::SimpleRequest;
use CONFIG;

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
    // unwrap hell
    // TODO make function safer
    pub fn generate_access_token() {
        let params = [
            ("refresh_token", REFRESH_TOKEN.lock().unwrap().to_string()),
            ("client_id", CLIENT_ID.to_string()),
            ("client_secret", CLIENT_SECRET.to_string()),
            ("grant_type", "refresh_token".to_string()),
        ];
        let client = reqwest::Client::new();

        let res: AccessToken = client.post("https://api.imgur.com/oauth2/token").form(&params).send().unwrap().json().unwrap();

        let mut access_token = ACCESS_TOKEN.lock().unwrap();
        let mut refresh_token = REFRESH_TOKEN.lock().unwrap();

        access_token.clear();
        access_token.insert_str(0, &*res.access_token);
        refresh_token.clear();
        refresh_token.insert_str(0, &*res.refresh_token);

        debug!("generate_access_token -> {}", access_token);
        debug!("generate_refresh_token -> {}", refresh_token);
    }

    pub fn account_images() -> Option<Value> {
        let mut headers = Headers::new();
        headers.set(Authorization(Bearer {
            token: ACCESS_TOKEN.lock().unwrap().to_owned(),
        }));
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
        headers.set(Authorization(Bearer {
            token: ACCESS_TOKEN.lock().unwrap().to_owned(),
        }));
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
        headers.set(Authorization(Bearer {
            token: ACCESS_TOKEN.lock().unwrap().to_owned(),
        }));
        let uri = format!("https://api.imgur.com/3/album/{}/images", get_current_album_id(guild_id).expect("No Album set"),);
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
            return request
                .clone()
                .headers({
                    let mut headers = request.get_headers().unwrap();
                    headers.set(Authorization(Bearer {
                        token: ACCESS_TOKEN.lock().unwrap().to_owned(),
                    }));
                    headers
                })
                .run();
        }
        return Some(result);
    } else {
        return None;
    }
}

pub fn set_album_id(album_id: &str, guild_id: i64) -> Result<(), Box<Error>> {
    pg_backend::execute_sql("UPDATE settings SET imgur_album_id = $1 WHERE guild_id = $2", &[&album_id, &guild_id])?;
    Ok(())
}

pub fn get_current_album_id(guild_id: i64) -> Result<String, Box<Error>> {
    let rows = pg_backend::query_sql("SELECT imgur_album_id FROM settings WHERE guild_id = $1", &[&guild_id])?;
    let mut album_id: String = String::new();
    for row in &rows {
        let album_id_row: String = row.get(0);
        debug!("Current Album: {}", album_id_row);
        album_id = album_id_row;
    }
    Ok(album_id)
}
