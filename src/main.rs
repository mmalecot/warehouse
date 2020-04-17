#[macro_use]
extern crate diesel;
#[macro_use]
extern crate diesel_migrations;

use crate::{
    core::{
        config::Config,
        error::{self, WarehouseResult},
        logger,
    },
    database::Pool,
    utils::{auth::AuthenticationService, path, regex::Regexes},
};
use actix_identity::{CookieIdentityPolicy, IdentityService};
use actix_web::{
    http::StatusCode,
    middleware::{errhandlers::ErrorHandlers, Logger},
    App, FromRequest, HttpServer,
};
use awmp::Parts;
use diesel::r2d2::ConnectionManager;
use log::{debug, info};
use std::{env, net::SocketAddr, process, time::Duration};
use tera::Tera;

mod core;
mod database;
mod service;
mod utils;

#[actix_rt::main]
async fn main() -> WarehouseResult {
    // Loads the configuration
    let config = Config::load()?;

    // Sets up the logger
    logger::setup(&config.logger)?;

    // Prints the start message
    info!(
        "Starting {name} {version} with PID {pid} in {directory}",
        name = env!("CARGO_PKG_NAME"),
        version = env!("CARGO_PKG_VERSION"),
        pid = process::id(),
        directory = env::current_dir()?.display()
    );

    // Prints the configuration
    debug!("{:?}", config.database);
    debug!("{:?}", config.logger);
    debug!("{:?}", config.server);
    debug!("{:?}", config.session);
    debug!("{:?}", config.ui);

    // Runs pending database migrations
    database::run_pending_migrations(&config.database.url)?;

    // Constructs the application
    let tera = Tera::new(&format!("{}/**/*", path::templates_dir().display()))?;
    let regexes = Regexes::load()?;
    let pool = Pool::builder()
        .connection_timeout(Duration::from_secs(config.database.pool_connection_timeout))
        .idle_timeout(Some(Duration::from_secs(config.database.pool_idle_timeout)))
        .max_lifetime(Some(Duration::from_secs(config.database.pool_max_lifetime)))
        .max_size(config.database.pool_max_size)
        .min_idle(Some(config.database.pool_min_idle))
        .build(ConnectionManager::new(config.database.url.clone()))?;
    let secret_key = base64::decode(&config.session.secret_key)?;
    let address = SocketAddr::new(config.server.ip_address, config.server.port);
    let workers = config.server.workers;

    // Starts the HTTP server
    info!("Starting server on {}", address);
    HttpServer::new(move || {
        App::new()
            .data(Parts::configure(|parts_config| {
                parts_config.with_file_limit(config.server.upload_limit)
            }))
            .data(config.clone())
            .data(tera.clone())
            .data(regexes.clone())
            .data(pool.clone())
            .wrap(Logger::new(&config.logger.access_format))
            .wrap(
                ErrorHandlers::new()
                    .handler(StatusCode::BAD_REQUEST, error::handle)
                    .handler(StatusCode::INTERNAL_SERVER_ERROR, error::handle)
                    .handler(StatusCode::NOT_FOUND, error::handle)
                    .handler(StatusCode::UNAUTHORIZED, error::handle),
            )
            .wrap(AuthenticationService)
            .wrap(IdentityService::new(
                CookieIdentityPolicy::new(&secret_key)
                    .name(config.session.cookie_name.clone())
                    .secure(config.session.cookie_secure),
            ))
            .configure(service::configure)
    })
    .workers(workers)
    .bind(&address)?
    .run()
    .await?;
    Ok(())
}
