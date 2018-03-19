use CONFIG;
use serde_json;
use std::io;
use futures::{Future, Stream};
use hyper::Client;
use hyper_tls::HttpsConnector;
use tokio_core::reactor::Core;
use hyper::{Method, Request};
use serde_json::Value;
use serenity::model::channel::Message;
use serenity::utils::Colour;
use serenity::prelude::Mutex;
use transient_hashmap::TransientHashMap;
use serenity::model::id::UserId;




lazy_static! {
    pub static ref YTS: Mutex<TransientHashMap<UserId, Vec<Value>>> = Mutex::new(TransientHashMap::new(30));
}

pub struct API {

}

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
                let mut core = Core::new().unwrap();
                let handle = core.handle();
                let client = Client::configure()
                    .connector(HttpsConnector::new(4, &handle).unwrap())
                    .build(&handle);

                let uri = format!(
                    "https://www.googleapis.com/youtube/v3/search?part=snippet&q={}&maxResults={}&key={}",
                    query, limit, token
                );
                let uri = &uri[..];
                debug!("{}", uri);

                let request = client
                    .request(Request::new(Method::Get, uri.parse().unwrap()))
                    .and_then(|res| {
                        debug!("GET: {}", res.status());

                        res.body().concat2().and_then(move |body| {
                            let v: Value = serde_json::from_slice(&body).map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;
                            Ok(v)
                        })
                    });

                let result = core.run(request).unwrap();

                let items = match result["items"].as_array() {
                    Some(array) => array.to_owned(),
                    None => Vec::new(),
                };
                debug!("{}", items.len());

                //Build instruction message
                let mut instruction: String = "Please select a track with the ".to_owned();
                instruction.push_str(&CONFIG.required.prefix);
                instruction.push_str("play command:");

                let _ = msg.channel_id.send_message(|m| {
                    m.content(&instruction).embed(|e| {
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
                        debug!("{}", &vid["snippet"]["title"]);
                    }
                    e
                    })
                });
                push(msg, items);
                debug!(
                    "################### {:?}",
                    YTS.lock().get(&msg.author.id).unwrap()
                );
            } else {
                let _ = msg.channel_id.say("Missing Youtube token!");
            }
        } else {
            let _ = msg.channel_id.say("Missing Youtube token!");
        }
    }
}
fn push(msg: &Message, vec: Vec<Value>) {
    YTS.lock().insert(msg.author.id, vec);
}
