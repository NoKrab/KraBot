use serde_json;
use serde_json::Value;
use std::collections::HashMap;
use std::fs::File;
use std::io::Read;

lazy_static! {
    static ref EMOTES_BTTV: EmotesAPI = {
        let mut file = File::open("assets/bttv_emotes.json").expect("Unable to open bttv_emotes.json file");
        let mut data = String::new();
        file.read_to_string(&mut data).unwrap();
        let v: Value = serde_json::from_str(&data).unwrap();

        if let Some(emotes) = v["emotes"].as_array() {
            debug!("deserialized  emotes: {:?}", emotes);
            let mut emotes_map: HashMap<String, Emote> = {
                let mut m = HashMap::new();
                for emote in emotes {
                    m.insert(
                        String::from(emote["code"].as_str().unwrap()),
                        Emote {
                            id: String::from(emote["id"].as_str().unwrap()),
                            code: String::from(emote["code"].as_str().unwrap()),
                        },
                    );
                }
                m
            };

            let emotes_api = EmotesAPI {
                url_cdn: String::from("https://cdn.betterttv.net/emote"),
                emotes: emotes_map,
            };

            return emotes_api;
        } else {
            panic!("Could not parse file bttv_emotes.json");
        }
    };
}

#[derive(Debug, Deserialize)]
struct EmotesAPI {
    url_cdn: String,
    emotes: HashMap<String, Emote>,
}

#[derive(Debug, Deserialize)]
struct Emote {
    id: String,
    code: String,
}

pub struct API {}

impl API {
    pub fn get_emote_uri(code: &str) -> Option<String> {
        if EMOTES_BTTV.emotes.contains_key(code) {
            if let Some(emote) = EMOTES_BTTV.emotes.get(code) {
                return Some(format!("{}/{}/2x", EMOTES_BTTV.url_cdn, emote.id));
            } else {
                return None;
            }
        } else {
            return None;
        }
    }
}
