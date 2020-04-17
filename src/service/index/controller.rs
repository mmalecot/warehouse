use crate::{core::error::WarehouseResult, utils::path};
use actix_files::NamedFile;
use actix_web::{http::header::LOCATION, HttpResponse, Result};

pub async fn serve_favicon() -> Result<NamedFile> {
    Ok(NamedFile::open(&format!(
        "{}/images/favicon.ico",
        path::static_files_dir().display()
    ))?)
}

pub async fn serve_index_page() -> WarehouseResult<HttpResponse> {
    Ok(HttpResponse::Found()
        .header(LOCATION, "/package/list")
        .finish())
}
