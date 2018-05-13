use CONFIG;
use serde_json::Value;
use serenity::model::channel::Message;
use serenity::prelude::Mutex;
use transient_hashmap::TransientHashMap;
use serenity::model::id::UserId;
use regex::Regex;
use hyper::{Method};
use util::network::request::request::SimpleRequest;



lazy_static! {
    pub static ref YTS: Mutex<TransientHashMap<UserId, Vec<Value>>> = Mutex::new(TransientHashMap::new(30));
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
            info!("Youtbe API token: {}", token);
            if !token.is_empty() {
                let limit = 5;
                let uri = format!(
                    "https://www.googleapis.com/youtube/v3/search?part=snippet&type=video&q={}&maxResults={}&key={}",
                    query, limit, token
                );

                let result = SimpleRequest::new().uri(uri).method(Method::Get).run().unwrap();

                let items = match result["items"].as_array() {
                    Some(array) => array.to_owned(),
                    None => Vec::new(),
                };

                //Build instruction message
                let mut instruction: String = "Please select a track with the ".to_string();
                instruction.push_str(&CONFIG.required.prefix);
                instruction.push_str("play command:");

                let _ = msg.channel_id.send_message(|m| {
                    let mut m = m.content(&instruction);
                    m.embed(|e| {
//                        e.author(|a| {
//                            a.name("KinG")
//                                .icon_url("http://i0.kym-cdn.com/photos/images/newsfeed/001/339/830/135.png")
//                        }).title("HALLO I BIMS EIN TITEL")
//                            .thumbnail("http://i0.kym-cdn.com/photos/images/newsfeed/001/326/935/569.png")
//                            .description("I BIMS EINE DESKRIPTION")
//                            .field("ok", "Hello World", false)
//                            .colour(Colour::dark_teal())
//                            .footer(|f| {
//                                f.icon_url("http://i0.kym-cdn.com/photos/images/newsfeed/001/331/733/acc.png")
//                                    .text("I BIMS DA FOOTER TECKST ECKS DEE")
//                            })
                   let mut e = e;

                    let mut count = 1;

                    //Add sone field for one video result
                    for vid in &items {
                        e = e.field(count, &vid["snippet"]["title"], false);
                        count += 1;
                    }
                    e
                    })
                });
                push(msg, items);
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
            url.push_str(&RE_LINKS.replace_all(&video["id"]["videoId"].to_string(), "").into_owned());
            info!("{}", url);
            return url;
        } else {
            return String::from("Search results are already purged");
        }
    }
}
fn push(msg: &Message, vec: Vec<Value>) {
    YTS.lock().insert(msg.author.id, vec);
}
