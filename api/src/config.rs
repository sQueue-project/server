use serde::Deserialize;

#[derive(Debug, Deserialize, Clone)]
pub struct Config {
    pub mysql_host: String,
    pub mysql_database: String,
    pub mysql_username: String,
    pub mysql_password: String,
    pub google_oauth_client_id: String,
    pub google_oauth_client_secret: String,
    pub google_api_key: String,
}

impl Config {
    pub fn from_env() -> Result<Self, envy::Error> {
        envy::from_env::<Self>()
    }
}