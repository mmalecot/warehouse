use crate::{
    core::{
        config::Config,
        error::{ImportPackageError, ReadPackageError, WarehouseError, WarehouseResult},
    },
    database::PooledConnection,
    service::{package::model::Package, repository::model::Repository, user::model::User},
    utils::{auth::Authentication, package, path},
    view,
};
use actix_files::NamedFile;
use actix_web::{
    http::header::LOCATION,
    web::{Data, Path, Query},
    HttpRequest, HttpResponse, Result,
};
use awmp::Parts;
use serde::Deserialize;
use std::fs::{self};

#[derive(Deserialize)]
#[serde(default)]
pub struct PackageListQuery {
    page: i32,
}

impl Default for PackageListQuery {
    fn default() -> PackageListQuery {
        PackageListQuery { page: 1 }
    }
}

#[derive(Deserialize)]
pub struct PackagePath {
    repository: String,
    architecture: String,
    name: String,
}

#[derive(Deserialize)]
pub struct PackageFilePath {
    repository: String,
    architecture: String,
    name: String,
    extension: String,
}

pub async fn delete_package(
    connection: PooledConnection,
    path: Path<PackagePath>,
) -> WarehouseResult<HttpResponse> {
    match Package::find_by_name_repository_and_architecture(
        &connection,
        &path.name,
        &path.repository,
        &path.architecture,
    )? {
        Some((package, repository, _)) => {
            package::remove_package_from_repository(
                &path.name,
                &path::repository_file(&path.repository, &path.architecture, &repository.extension),
            )?;
            let package_path = path::package_file(
                &path.repository,
                &path.architecture,
                &path.name,
                &package.extension,
            );
            if package_path.exists() {
                fs::remove_file(&package_path)?;
            }
            package.delete_versions(&connection)?;
            package.delete_dependencies(&connection)?;
            package.delete_files(&connection)?;
            package.delete(&connection)?;
            Ok(HttpResponse::Ok().into())
        }
        None => Ok(HttpResponse::NotFound().into()),
    }
}

pub async fn fetch_package(
    auth: Authentication,
    connection: PooledConnection,
    path: Path<PackagePath>,
    request: HttpRequest,
) -> WarehouseResult<HttpResponse> {
    match Package::find_by_name_repository_and_architecture(
        &connection,
        &path.name,
        &path.repository,
        &path.architecture,
    )? {
        Some(package) => view!(&request, "route/package/detail", [
            "user" => &auth.user(),
            "package" => &package,
            "files" => &package.0.list_files(&connection)?,
            "dependencies" => &package.0.list_dependencies(&connection)?,
            "versions" => &package.0.list_versions(&connection)?
        ]),
        None => Ok(HttpResponse::NotFound().into()),
    }
}

pub async fn handle_import_package_post(
    connection: PooledConnection,
    parts: Parts,
    request: HttpRequest,
    user: User,
) -> WarehouseResult<HttpResponse> {
    if let Err(error) = package::import_package(&connection, parts, &user) {
        let error = match error {
            ImportPackageError::MultipartError(awmp::Error::FileTooLarge { limit, .. }) => {
                format!("File too large. Limited to {} bytes.", limit)
            }
            ImportPackageError::ReadPackageError(ReadPackageError::AlpmError(
                alpm::Error::PkgInvalid,
            ))
            | ImportPackageError::ReadPackageError(ReadPackageError::AlpmError(
                alpm::Error::PkgOpen,
            )) => String::from("Invalid package format."),
            ImportPackageError::ReadPackageError(ReadPackageError::UnsupportedFileType) => {
                String::from("Unsupported file type.")
            }
            ImportPackageError::OlderPackageVersion { old, new } => format!(
                "Package already exists in a more recent version. {} <= {}.",
                new, old
            ),
            ImportPackageError::UnauthorizedUpdate => {
                String::from("You are not the maintainer of the package.")
            }
            _ => return Err(error.into()),
        };
        view!(&request, "route/package/import", [
            "user" => &user,
            "repositories" => &Repository::list(&connection)?,
            "error" => &error
        ])
    } else {
        Ok(HttpResponse::Found().header(LOCATION, "/").finish())
    }
}

pub async fn serve_import_package_page(
    connection: PooledConnection,
    auth: Authentication,
    request: HttpRequest,
) -> WarehouseResult<HttpResponse> {
    view!(&request, "route/package/import", [
        "user" => &auth.user(),
        "repositories" => &Repository::list(&connection)?
    ])
}

pub async fn serve_package_archive(path: Path<PackageFilePath>) -> Result<NamedFile> {
    Ok(NamedFile::open(&path::package_file(
        &path.repository,
        &path.architecture,
        &path.name,
        &path.extension,
    ))?)
}

pub async fn serve_package_list_page(
    auth: Authentication,
    config: Data<Config>,
    connection: PooledConnection,
    query: Query<PackageListQuery>,
    request: HttpRequest,
) -> WarehouseResult<HttpResponse> {
    if query.page > 0 {
        let offset = (i64::from(query.page) - 1) * i64::from(config.ui.paging_num);
        let limit = i64::from(config.ui.paging_num);
        let packages = Package::list(&connection, offset, limit)?;
        let page_count =
            (Package::count(&connection)? as f64 / f64::from(config.ui.paging_num)).ceil() as i64;
        view!(&request, "route/package/list", [
            "user" => &auth.user(),
            "page" => &query.page,
            "page_count" => &page_count,
            "packages" => &packages
        ])
    } else {
        Err(WarehouseError::InvalidPathData)
    }
}

pub async fn serve_repository_database(path: Path<PackageFilePath>) -> Result<NamedFile> {
    Ok(NamedFile::open(&path::repository_file(
        &path.repository,
        &path.architecture,
        &path.extension,
    ))?)
}
