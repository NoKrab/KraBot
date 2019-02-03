use super::models::AccessToken;
// use hyper::header::{Authorization, Bearer, ContentType, Headers};
// use hyper::Method;
use reqwest::header::{HeaderMap, HeaderValue, CONTENT_TYPE, USER_AGENT};
use serde_derive;
use serde_json::Value;
use std::error::Error;
use std::sync::Mutex;
// use util::network::request::request::SimpleRequest;
use rsbot_lib::models::*;
use CONFIG;
use DIESEL_PG;

use futures::Future;
use reqwest::async::{Client, Response};

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
    static ref CLIENT: reqwest::Client = reqwest::Client::new();
}

type Result<T> = std::result::Result<T, Box<Error>>;

pub struct Account {}
pub struct Comment {}
pub struct Album {}
pub struct Gallery {}
pub struct Image {}

impl Account {
    // unwrap hell
    // TODO make function safer
    fn async_generate_access_token() -> impl Future<Item = (), Error = ()> {
        let json = |mut res: Response| res.json::<AccessToken>();
        let params = [
            ("refresh_token", REFRESH_TOKEN.lock().unwrap().to_string()),
            ("client_id", CLIENT_ID.to_string()),
            ("client_secret", CLIENT_SECRET.to_string()),
            ("grant_type", "refresh_token".to_string()),
        ];

        Client::new()
            .post("https://api.imgur.com/oauth2/token")
            .form(&params)
            .send()
            .and_then(json)
            .map_err(|err| {
                error!("{}", err);
            })
            .map(|res| {
                debug!("{:#?}", res);
                let mut access_token = ACCESS_TOKEN.lock().unwrap();
                let mut refresh_token = REFRESH_TOKEN.lock().unwrap();
                access_token.clear();
                access_token.insert_str(0, &*res.access_token);
                refresh_token.clear();
                refresh_token.insert_str(0, &*res.refresh_token);
                debug!("generate_access_token -> {}", access_token);
                debug!("generate_refresh_token -> {}", refresh_token);
            })
    }
    pub fn generate_access_token() {
        tokio::run(Account::async_generate_access_token());
    }

    pub fn account_images() -> Option<Value> {
        Account::get_data("https://api.imgur.com/3/account/me/images").unwrap()
        // let mut headers = Headers::new();
        // headers.set(Authorization(Bearer {
        //     token: ACCESS_TOKEN.lock().unwrap().to_owned(),
        // }));
        // let request = SimpleRequest::new().headers(headers).uri("https://api.imgur.com/3/account/me/images".to_string()).method(Method::Get);
        // if let Some(result) = make_imgur_request(request) {
        //     if result["success"] == true {
        //         return Some(result["data"].to_owned());
        //     } else {
        //         return None;
        //     }
        // } else {
        //     return None;
        // }
    }

    // converts response to string which then gets turned into a value, we only want the data part
    fn return_data(mut response: reqwest::Response) -> Result<serde_json::Value> {
        let v: serde_json::Value = serde_json::from_str::<serde_json::Value>(&*response.text()?)?["data"].to_owned();
        Ok(v)
    }

    fn get_data(uri: &str) -> Result<Option<serde_json::Value>> {
        let mut response = CLIENT.get(uri).bearer_auth(ACCESS_TOKEN.lock()?).send()?;
        if response.status().is_success() {
            let v: serde_json::Value = serde_json::from_str::<serde_json::Value>(&*response.text()?)?["data"].to_owned();
            Ok(Some(v))
        } else if response.status().is_client_error() {
            Account::generate_access_token();
            // this can produce nasty loops
            return Account::get_data(uri);
        } else {
            // Err(Box::from(format!("Something else happened. Status: {:?}", response.status())))
            Ok(None)
        }
    }

    pub fn albums() -> Option<Value> {
        // let resp = CLIENT.get("https://api.imgur.com/3/account/me/albums").bearer_auth(ACCESS_TOKEN.lock().unwrap()).send().unwrap();
        // if resp.status().is_success() {
        //     return Some(Account::return_data(resp).unwrap());
        // } else if resp.status().is_client_error() {
        //     Account::generate_access_token();
        //     return Account::albums();
        // } else {
        //     error!("Something else happened. Status: {:?}", resp.status());
        //     return None;
        // }
        Account::get_data("https://api.imgur.com/3/account/me/albums").unwrap()
    }
}

impl Album {
    pub fn album_images(guild_id: i64) -> Option<Value> {
        let uri = &*format!("https://api.imgur.com/3/album/{}/images", get_current_album_id(guild_id).expect("No Album set"),);
        Account::get_data(uri).unwrap()
        // let mut headers = Headers::new();
        // headers.set(Authorization(Bearer {
        //     token: ACCESS_TOKEN.lock().unwrap().to_owned(),
        // }));
        // let request = SimpleRequest::new().headers(headers).uri(uri).method(Method::Get);
        // if let Some(result) = make_imgur_request(request) {
        //     if result["success"] == true {
        //         return Some(result["data"].to_owned());
        //     } else {
        //         return None;
        //     }
        // } else {
        //     return None;
        // }
    }
}

// fn make_imgur_request(request: SimpleRequest) -> Option<Value> {
//     if let Some(result) = request.clone().run() {
//         if result["status"] == 403 {
//             info!("Requesting new token...");
//             Account::generate_access_token();
//             return request
//                 .clone()
//                 .headers({
//                     let mut headers = request.get_headers().unwrap();
//                     headers.set(Authorization(Bearer {
//                         token: ACCESS_TOKEN.lock().unwrap().to_owned(),
//                     }));
//                     headers
//                 })
//                 .run();
//         }
//         return Some(result);
//     } else {
//         return None;
//     }
// }

pub fn set_album_id(album_id: &str, guild_id: i64) -> Result<()> {
    let mut guild = DIESEL_PG.get_guild(guild_id as u64)?;
    guild.imgur_album_id = Some(album_id.to_owned());
    DIESEL_PG.update_guild(&guild);
    Ok(())
}

pub fn get_current_album_id(guild_id: i64) -> Result<String> {
    let guild = DIESEL_PG.get_guild(guild_id as u64)?;
    if let Some(s) = guild.imgur_album_id {
        return Ok(s);
    } else {
        return Err(Box::from("No Album set"));
    }
}
