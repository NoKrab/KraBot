#[derive(Serialize, Deserialize, Debug)]
pub struct Default {
    pub url: String,
    pub width: u64,
    pub height: u64,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Id {
    pub kind: String,
    #[serde(rename = "videoId")]
    pub video_id: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct PageInfo {
    #[serde(rename = "totalResults")]
    pub total_results: u64,
    #[serde(rename = "resultsPerPage")]
    pub results_per_page: u64,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct SearchResult {
    pub kind: String,
    pub etag: String,
    #[serde(rename = "nextPageToken")]
    pub next_page_token: String,
    #[serde(rename = "regionCode")]
    pub region_code: String,
    #[serde(rename = "pageInfo")]
    pub page_info: PageInfo,
    pub items: Vec<Items>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Snippet {
    #[serde(rename = "publishedAt")]
    pub published_at: String,
    #[serde(rename = "channelId")]
    pub channel_id: String,
    pub title: String,
    pub description: String,
    pub thumbnails: Thumbnails,
    #[serde(rename = "channelTitle")]
    pub channel_title: String,
    #[serde(rename = "liveBroadcastContent")]
    pub live_broadcast_content: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Thumbnails {
    pub default: Default,
    pub medium: Default,
    pub high: Default,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Items {
    pub kind: String,
    pub etag: String,
    pub id: Id,
    pub snippet: Snippet,
}
