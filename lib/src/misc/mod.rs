pub struct Metadata {
    pub version: &'static str,
    pub git: String,
    pub date: &'static str,
}

impl Default for Metadata {
    fn default() -> Self {
        let no_val = "No Data";
        Self {
            version: no_val,
            git: "No Data".to_owned(),
            date: no_val,
        }
    }
}
