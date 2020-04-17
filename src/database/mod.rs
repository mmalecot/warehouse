use crate::core::error::{RunPendingMigrationsError, WarehouseError};
use actix_web::{dev::Payload, web::Data, FromRequest, HttpRequest};
use futures::future::{self, Ready};
use log::info;
use std::{fs, ops::Deref, path::Path};

pub mod schema;

#[cfg(feature = "mysql")]
pub type Connection = diesel::MysqlConnection;
#[cfg(feature = "postgres")]
pub type Connection = diesel::PgConnection;
#[cfg(feature = "sqlite")]
pub type Connection = diesel::SqliteConnection;

pub type Pool = diesel::r2d2::Pool<diesel::r2d2::ConnectionManager<Connection>>;

pub struct PooledConnection {
    inner: r2d2::PooledConnection<diesel::r2d2::ConnectionManager<Connection>>,
}

impl Deref for PooledConnection {
    type Target = r2d2::PooledConnection<diesel::r2d2::ConnectionManager<Connection>>;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl FromRequest for PooledConnection {
    type Error = WarehouseError;
    type Future = Ready<Result<Self, Self::Error>>;
    type Config = ();

    fn from_request(request: &HttpRequest, _: &mut Payload) -> Self::Future {
        if let Some(pool) = request.app_data::<Data<Pool>>() {
            match pool.get() {
                Ok(connection) => future::ok(PooledConnection { inner: connection }),
                Err(error) => future::err(error.into()),
            }
        } else {
            future::err(WarehouseError::AppDataNotFound(String::from("Pool")))
        }
    }
}

embed_migrations!("database/migrations");

pub fn run_pending_migrations(database_url: &str) -> Result<(), RunPendingMigrationsError> {
    if cfg!(feature = "sqlite") {
        if let Some(parent) = Path::new(database_url).parent() {
            fs::create_dir_all(parent)?;
        }
    }
    let connection: Connection = diesel::Connection::establish(database_url)?;
    let mut output = Vec::new();
    embedded_migrations::run_with_output(&connection, &mut output)?;
    let output = String::from_utf8(output)?;
    for line in output.lines() {
        info!("{}", line);
    }
    Ok(())
}
