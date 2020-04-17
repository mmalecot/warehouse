use crate::view;
use actix_web::{
    dev::ServiceResponse, http::StatusCode, middleware::errhandlers::ErrorHandlerResponse,
    HttpResponse, ResponseError,
};
use derive_more::{Display, From};
use log::error;

pub type WarehouseResult<T = ()> = Result<T, WarehouseError>;

#[derive(Debug, Display, From)]
pub enum ImportPackageError {
    #[display(fmt = "{}", _0)]
    DieselError(diesel::result::Error),

    #[display(fmt = "File not found")]
    #[from(ignore)]
    FileNotFound,

    #[display(fmt = "{}", _0)]
    Io(std::io::Error),

    #[display(fmt = "{}", _0)]
    MultipartError(awmp::Error),

    #[display(
        fmt = "Package already exists in a more recent version. {} <= {}",
        new,
        old
    )]
    #[from(ignore)]
    OlderPackageVersion { old: String, new: String },

    #[display(fmt = "{}", _0)]
    ReadPackageError(ReadPackageError),

    #[display(fmt = "Repository {} not found", _0)]
    #[from(ignore)]
    RepositoryNotFound(String),

    #[display(fmt = "{}", _0)]
    TempFilePersistError(tempfile::PersistError),

    #[display(fmt = "Text field {} not found", _0)]
    #[from(ignore)]
    TextFieldNotFound(String),

    #[display(fmt = "Unauthorized update")]
    #[from(ignore)]
    UnauthorizedUpdate,
}

#[derive(Debug, Display, From)]
pub enum LoadConfigError {
    #[display(fmt = "{}", _0)]
    AddrParseError(std::net::AddrParseError),

    #[display(fmt = "{}", _0)]
    Io(std::io::Error),

    #[display(fmt = "{}", _0)]
    Infallible(std::convert::Infallible),

    #[display(fmt = "{}", _0)]
    ParseBoolError(std::str::ParseBoolError),

    #[display(fmt = "{}", _0)]
    ParseIntError(std::num::ParseIntError),

    #[display(fmt = "{}", _0)]
    ParseLevelError(log::ParseLevelError),

    #[display(fmt = "{}", _0)]
    TomlDeserializeError(toml::de::Error),
}

#[derive(Debug, Display, From)]
pub enum ReadPackageError {
    #[display(fmt = "{}", _0)]
    AlpmError(alpm::Error),

    #[display(fmt = "{}", _0)]
    Io(std::io::Error),

    #[display(fmt = "Unsupported file type")]
    #[from(ignore)]
    UnsupportedFileType,
}

#[derive(Debug, Display, From)]
pub enum RunPendingMigrationsError {
    #[display(fmt = "{}", _0)]
    DieselConnectionError(diesel::ConnectionError),

    #[display(fmt = "{}", _0)]
    DieselMigrationsError(diesel_migrations::RunMigrationsError),

    #[display(fmt = "{}", _0)]
    FromUtf8Error(std::string::FromUtf8Error),

    #[display(fmt = "{}", _0)]
    Io(std::io::Error),
}

#[derive(Debug, Display, From)]
pub enum SetupLoggerError {
    #[display(fmt = "{}", _0)]
    Io(std::io::Error),

    #[display(fmt = "{}", _0)]
    SetLoggerError(log::SetLoggerError),
}

#[derive(Debug, Display, From)]
pub enum WarehouseError {
    #[display(fmt = "{} not found in app data", _0)]
    #[from(ignore)]
    AppDataNotFound(String),

    #[display(fmt = "{}", _0)]
    Base64DecodeError(base64::DecodeError),

    #[display(fmt = "{}", _0)]
    BcryptError(bcrypt::BcryptError),

    #[display(fmt = "{}", _0)]
    DieselError(diesel::result::Error),

    #[display(fmt = "Invalid form data")]
    #[from(ignore)]
    InvalidFormData,

    #[display(fmt = "{}", _0)]
    Io(std::io::Error),

    #[display(fmt = "Invalid path data")]
    #[from(ignore)]
    InvalidPathData,

    #[display(fmt = "{}", _0)]
    ImportPackageError(ImportPackageError),

    #[display(fmt = "{}", _0)]
    LoadConfigError(LoadConfigError),

    #[display(fmt = "{}", _0)]
    R2d2Error(r2d2::Error),

    #[display(fmt = "{}", _0)]
    RegexError(regex::Error),

    #[display(fmt = "{}", _0)]
    RunPendingMigrationsError(RunPendingMigrationsError),

    #[display(fmt = "{}", _0)]
    SetupLoggerError(SetupLoggerError),

    #[display(fmt = "{}", _0)]
    TeraError(tera::Error),
}

impl ResponseError for WarehouseError {
    fn error_response(&self) -> HttpResponse {
        HttpResponse::new(match *self {
            WarehouseError::ImportPackageError(ImportPackageError::FileNotFound)
            | WarehouseError::ImportPackageError(ImportPackageError::RepositoryNotFound(..))
            | WarehouseError::ImportPackageError(ImportPackageError::TextFieldNotFound(..))
            | WarehouseError::InvalidFormData
            | WarehouseError::InvalidPathData => StatusCode::BAD_REQUEST,
            _ => StatusCode::INTERNAL_SERVER_ERROR,
        })
    }
}

pub fn handle<B>(service: ServiceResponse<B>) -> actix_web::Result<ErrorHandlerResponse<B>> {
    if let Some(error) = service.response().error() {
        if service.status().is_server_error() {
            error!("Error in response: {:?}", error);
        }
    }
    let response = (view!(service.request(), "misc/error", ["status" => &service.status().as_u16()])
        as WarehouseResult<HttpResponse>)?;
    Ok(ErrorHandlerResponse::Response(
        service.into_response(response.into_body()),
    ))
}
