pub struct Metadata {
    pub version: &'static str,
    pub git: &'static str,
    pub date: &'static str,
}

impl Default for Metadata {
    fn default() -> Self {
        let no_val = "No Data";
        Self {
            version: no_val,
            git: no_val,
            date: no_val,
        }
    }
}
