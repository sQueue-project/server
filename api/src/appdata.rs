use std::sync::Arc;
use dal::Mysql;
use crate::config::Config;
use thiserror::Error;

pub type WebData = actix_web::web::Data<Arc<AppData>>;

#[derive(Debug)]
pub struct AppData {
    pub config: Config,
    pub dal: Mysql
}

#[derive(Debug, Error)]
pub enum AppDataError {
    #[error("Dal error: {0}")]
    Dal(#[from] dal::Error)
}

impl AppData {
    pub fn new(config: Config) -> Result<Self, AppDataError> {
        let dal = Mysql::new(&config.mysql_host, &config.mysql_database, &config.mysql_username, &config.mysql_password)?;
        Ok(Self {
            config,
            dal
        })
    }

    pub fn migrate(&self) -> Result<(), AppDataError> {
        self.dal.migrate()?;
        Ok(())
    }
}