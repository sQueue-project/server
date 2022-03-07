use std::collections::HashMap;
use std::sync::Arc;
use parking_lot::Mutex;
use dal::Mysql;
use crate::config::Config;
use thiserror::Error;
use dal::uuid::Uuid;
use crate::services::sse::AMBroadcaster;

pub type WebData = actix_web::web::Data<Arc<AppData>>;

#[derive(Debug)]
pub struct AppData {
    pub config: Config,
    pub dal: Mysql,
    pub sse_track_list: Mutex<HashMap<Uuid, AMBroadcaster>>,
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
            dal,
            sse_track_list: Mutex::new(HashMap::new()),
        })
    }

    pub fn migrate(&self) -> Result<(), AppDataError> {
        self.dal.migrate()?;
        Ok(())
    }
}