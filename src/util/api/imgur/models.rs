#[derive(Serialize, Deserialize, Debug)]
pub struct AccessToken {
    pub access_token: String,
    pub expires_in: u64,
    pub token_type: String,
    pub scope: String,
    pub refresh_token: String,
    pub account_id: u64,
    pub account_username: String,
}
