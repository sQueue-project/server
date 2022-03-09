use std::sync::Arc;
use actix_web::{App, HttpServer, web};
use tracing::{debug, info, trace};
use tracing_actix_web::TracingLogger;
use crate::appdata::AppData;

mod config;
mod appdata;
mod services;
mod error;
mod apis;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    configure_tracing();
    info!("Starting {} v{}", env!("CARGO_PKG_NAME"), env!("CARGO_PKG_VERSION"));

    info!("Reading config");
    let config = config::Config::from_env().expect("Reading configuration from environmental variables");

    debug!("Creating appdata");
    let appdata = AppData::new(config).expect("Creating AppData");

    info!("Applying database migrations");
    appdata.migrate().expect("Applying migrations");

    debug!("Configuring Actix");
    let appdata_arc = Arc::new(appdata);
    let server = HttpServer::new(move || App::new()
        .app_data(web::Data::new(appdata_arc.clone()))
        .wrap(TracingLogger::default())
        .service(web::scope("/api/v1")
            .route("/room/create", web::post().to(services::room::create::create))
            .route("/room/get/{uuid}", web::get().to(services::room::get::get))
            .route("/room/join", web::post().to(services::room::join::join))
            .route("/room/leave", web::post().to(services::room::leave::leave))
            .route("/room/members", web::get().to(services::room::members::members))
            .route("/tracks/sse-list/{uuid}", web::get().to(services::tracks::sse_list::sse_list))
            .route("/tracks/list/{uuid}", web::get().to(services::tracks::list::list))
            .route("/tracks/add", web::post().to(services::tracks::add::add))
            .route("/tracks/find/youtube", web::post().to(services::tracks::find::youtube::youtube))
            .route("/tracks/find/search", web::get().to(services::tracks::find::search::search))
            .route("/user/get", web::get().to(services::user::get::get))))
        .bind("[::]:8080")?
        .run();

    info!("Started");
    server.await
}

fn configure_tracing() {
    let subscriber = tracing_subscriber::FmtSubscriber::builder()
        .compact()
        .with_max_level(tracing::Level::DEBUG)
        .finish();

    tracing::subscriber::set_global_default(subscriber)
        .expect("Setting default tracing subscriber");

    debug!("Tracing configured");

    info!("Active on INFO level");
    debug!("Active on DEBUG level");
    trace!("Active on TRACE level");
}