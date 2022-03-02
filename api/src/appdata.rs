use crate::config::Config;

pub type WebData = actix_web::web::Data<AppData>;

pub struct AppData {
    pub config: Config,
}