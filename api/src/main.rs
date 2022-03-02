use tracing::info;

mod config;
mod appdata;
mod services;
mod error;

fn main() {
    configure_tracing();
    let config = config::Config::from_env().expect("Reading configuration from environmental variables");


    info!("Test");
}

fn configure_tracing() {
    let subscriber = tracing_subscriber::FmtSubscriber::builder()
        .with_max_level(tracing::Level::TRACE)
        .finish();

    tracing::subscriber::set_global_default(subscriber)
        .expect("Setting default tracing subscriber");
}