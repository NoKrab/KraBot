use super::models::{Items, SearchResult};
use regex::Regex;
use reqwest::get;
use serenity::model::channel::Message;
use serenity::model::id::UserId;
use serenity::prelude::Mutex;
use transient_hashmap::TransientHashMap;
use CONFIG;

lazy_static! {
    pub static ref YTS: Mutex<TransientHashMap<UserId, Vec<Items>>> = Mutex::new(TransientHashMap::new(30));
    ///Compiling Regex only one time for better performance
    /// Regex from https://stackoverflow.com/questions/2742813/how-to-validate-youtube-video-ids
    static ref RE_LINKS: Regex = Regex::new(r"[^a-zA-Z0-9_-]").unwrap();
}

pub struct API {}

impl API {
    pub fn youtube_search(query: String, msg: &Message) {
        {
            &*YTS.lock().prune();
        }
        // Since Option is either None or Some, Some may just contain an empty String
        if let Some(ref token) = CONFIG.optional.youtube_token {
            let token = token.to_owned();
            debug!("Youtbe API token: {}", token);
            if !token.is_empty() {
                let limit = 5;
                let uri = format!("https://www.googleapis.com/youtube/v3/search?part=snippet&type=video&q={}&maxResults={}&key={}", query, limit, token);
                let uri: &str = &*uri;

                // this might get messy
                // TODO make function safe to use, in case a key is missing from the response
                // this WILL CRASH
                let res: SearchResult = get(uri).unwrap().json().unwrap();

                let search_result = res.items;

                //Build instruction message
                let mut instruction: String = "Please select a track with the ".to_string();
                instruction.push_str(&CONFIG.required.prefix);
                instruction.push_str("play command:");

                let _ = msg.channel_id.send_message(|m| {
                    let mut m = m.content(&instruction);
                    m.embed(|e| {
                        let mut e = e;

                        let mut count = 1;

                        for video in &search_result {
                            e = e.field(count, &video.snippet.title, false);
                            count += 1;
                        }
                        e
                    })
                });
                push(msg, search_result);
            } else {
                let _ = msg.channel_id.say("Missing Youtube token!");
            }
        } else {
            let _ = msg.channel_id.say("Missing Youtube token!");
        }
    }

    pub fn get_url(user_id: &UserId, index: usize) -> String {
        let mut yts = YTS.lock();
        yts.prune();
        if yts.contains_key(user_id) {
            let mut url = String::from("https://www.youtube.com/watch?v=").to_owned();
            let vec = yts.get(&user_id).unwrap();
            let video = &vec.get(index).unwrap();
            url.push_str(&RE_LINKS.replace_all(&video.id.video_id, "").into_owned());
            info!("{}", url);
            return url;
        } else {
            return String::from("Search results are already purged");
        }
    }
}
fn push(msg: &Message, vec: Vec<Items>) {
    YTS.lock().insert(msg.author.id, vec);
}
