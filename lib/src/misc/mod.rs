pub struct Metadata {
    pub version: String,
    pub git: String,
    pub date: String,
}

impl Default for Metadata {
    fn default() -> Self {
        Self {
            version: "No Data".to_string(),
            git: "No Data".to_string(),
            date: "No Data".to_string(),
        }
    }
}
