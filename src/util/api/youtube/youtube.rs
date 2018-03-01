use CONFIG;
use super::super::super::super::commands::voice::YTS;
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

// TODO make fn work
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

            //    let items = result["items"].as_array();
            //    debug!("{:#?}", result);
            //    debug!("{}", result["etag"]);
            //    debug!("{}", result["items"]);
            debug!("{}", items.len());
            //debug!("{:#?}", items);
            super::super::super::super::commands::voice::push(msg, items);
            debug!(
                "################### {:?}",
                YTS.lock().get(&msg.author.id).unwrap()
            );

            // m is a create_message struct not a Message struct!
            // e is a create_embed struct
            let _ = msg.channel_id.send_message(|m| {
                m.content("KONTENT").embed(|e| {
                    e.author(|a| {
                        a.name("KinG")
                            .icon_url("http://i0.kym-cdn.com/photos/images/newsfeed/001/339/830/135.png")
                    }).title("HALLO I BIMS EIN TITEL")
                        .thumbnail("http://i0.kym-cdn.com/photos/images/newsfeed/001/326/935/569.png")
                        .description("I BIMS EINE DESKRIPTION")
                        .colour(Colour::dark_teal())
                        .footer(|f| {
                            f.icon_url("http://i0.kym-cdn.com/photos/images/newsfeed/001/331/733/acc.png")
                                .text("I BIMS DA FOOTER TECKST ECKS DEE")
                        })
                })
            });
        } else {
            let _ = msg.channel_id.say("Missing Youtube token!");
        }
    } else {
        let _ = msg.channel_id.say("Missing Youtube token!");
    }
}
