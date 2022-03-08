pub mod google_oauth2;
pub mod youtube;

use lazy_static::lazy_static;

lazy_static! {
    static ref CLIENT: reqwest::Client = reqwest::Client::new();
}