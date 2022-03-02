use serde::Deserialize;

#[derive(Debug, Deserialize, Clone)]
pub struct Config {
    pub mysql_host: String,
    pub mysql_database: String,
    pub mysql_username: String,
    pub mysql_password: String,
}


impl Config {
    pub fn from_env() -> Result<Self, envy::Error> {
        envy::from_env::<Self>()
    }
}