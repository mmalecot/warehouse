use crate::{
    core::error::{ImportPackageError, ReadPackageError, WarehouseResult},
    database::PooledConnection,
    service::{
        package::model::{Dependency, File, Package, Version},
        repository::model::Repository,
        user::model::User,
    },
    utils::path,
};
use alpm::{Alpm, SigLevel};
use awmp::Parts;
use chrono::{NaiveDateTime, Utc};
use diesel::Connection;
use std::{
    fs,
    io::Read,
    path::Path,
    process::{Command, Stdio},
};
use tempfile::NamedTempFile;
use uuid::Uuid;

pub struct PackageInfo {
    pub name: String,
    pub version: String,
    pub description: String,
    pub architecture: String,
    pub url: String,
    pub licenses: Vec<String>,
    pub dependencies: Vec<String>,
    pub compressed_size: i64,
    pub installed_size: i64,
    pub build_date: NaiveDateTime,
    pub files: Vec<(String, i64)>,
    pub extension: String,
}

impl PackageInfo {
    pub fn from_file<P: AsRef<Path>>(path: P) -> Result<PackageInfo, ReadPackageError> {
        let extension = find_extension(&path)?;
        let alpm = Alpm::new("/", "/var/lib/pacman")?;
        let path = path.as_ref().to_owned().display().to_string();
        let package = alpm.pkg_load(&path, true, SigLevel::NONE)?;
        Ok(PackageInfo {
            name: package.name().to_string(),
            version: package.version().to_string(),
            description: package.desc().to_string(),
            architecture: package.arch().to_string(),
            url: package.url().to_string(),
            licenses: package.licenses().map(|elem| elem.to_string()).collect(),
            dependencies: package.depends().map(|elem| elem.to_string()).collect(),
            compressed_size: package.size(),
            installed_size: package.isize(),
            build_date: NaiveDateTime::from_timestamp(package.build_date(), 0),
            files: package
                .files()
                .files()
                .iter()
                .map(|elem| (elem.name().to_string(), elem.size()))
                .filter(|(filename, _)| !filename.ends_with('/'))
                .collect(),
            extension: format!("pkg.tar.{}", extension),
        })
    }
}

pub fn add_package_to_repository(
    package_path: &Path,
    repository_path: &Path,
) -> Result<(), ImportPackageError> {
    Command::new("repo-add")
        .arg("--remove")
        .arg("--quiet")
        .arg(repository_path)
        .arg(package_path)
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()?;
    Ok(())
}

pub fn create_or_update_package(
    connection: &PooledConnection,
    repository: &Repository,
    user: &User,
    info: &PackageInfo,
) -> Result<(), ImportPackageError> {
    if let Some((mut package, _, _)) = Package::find_by_name_repository_and_architecture(
        connection,
        &info.name,
        &repository.name,
        &info.architecture,
    )? {
        if user.admin || package.maintainer_id == user.id {
            let old_version = alpm::Version::new(&package.version);
            let new_version = alpm::Version::new(&info.version);
            if new_version > old_version {
                update_package(connection, user, info, &mut package)
            } else {
                Err(ImportPackageError::OlderPackageVersion {
                    old: old_version.to_string(),
                    new: new_version.to_string(),
                })
            }
        } else {
            Err(ImportPackageError::UnauthorizedUpdate)
        }
    } else {
        create_package(connection, repository, user, info)
    }
}

pub fn create_package(
    connection: &PooledConnection,
    repository: &Repository,
    user: &User,
    info: &PackageInfo,
) -> Result<(), ImportPackageError> {
    let now = Utc::now().naive_utc();
    let package = Package {
        id: Uuid::new_v4().to_string(),
        creation_date: now,
        modification_date: now,
        name: info.name.clone(),
        version: info.version.clone(),
        description: info.description.clone(),
        url: info.url.clone(),
        build_date: info.build_date,
        compressed_size: info.compressed_size,
        installed_size: info.installed_size,
        architecture: info.architecture.clone(),
        license: info.licenses.join(" "),
        extension: info.extension.clone(),
        repository_id: repository.id.clone(),
        maintainer_id: user.id.clone(),
    };
    package.create(connection)?;
    for dependency in &info.dependencies {
        let dependency = Dependency {
            id: Uuid::new_v4().to_string(),
            name: dependency.clone(),
            package_id: package.id.clone(),
        };
        dependency.create(connection)?;
    }
    for (filename, size) in &info.files {
        let file = File {
            id: Uuid::new_v4().to_string(),
            name: filename.clone(),
            size: *size,
            package_id: package.id.clone(),
        };
        file.create(connection)?;
    }
    let version = Version {
        id: Uuid::new_v4().to_string(),
        creation_date: now,
        version: info.version.clone(),
        maintainer_id: user.id.clone(),
        package_id: package.id,
    };
    version.create(connection)?;
    Ok(())
}

pub fn find_extension<P: AsRef<Path>>(path: P) -> Result<String, ReadPackageError> {
    const LIMIT: usize = 7;
    let file = fs::File::open(&path)?;
    let mut buffer = [0; LIMIT];
    let mut handle = file.take(LIMIT as u64);
    if handle.read(&mut buffer)? < LIMIT {
        Err(ReadPackageError::UnsupportedFileType)
    } else {
        match buffer {
            [0x1F, 0x9D, ..] => Ok(String::from("Z")),
            [0x1F, 0x8B, ..] => Ok(String::from("gz")),
            [0x42, 0x5A, 0x68, ..] => Ok(String::from("bz2")),
            [0x28, 0xB5, 0x2F, 0xFD, ..] => Ok(String::from("zst")),
            [0x4C, 0x5A, 0x49, 0x50, ..] => Ok(String::from("lz")),
            [0x04, 0x22, 0x4D, 0x18, ..] => Ok(String::from("lz4")),
            [0xFD, 0x37, 0x7A, 0x58, 0x5A, 0x00, 0x00] => Ok(String::from("xz")),
            _ => Err(ReadPackageError::UnsupportedFileType),
        }
    }
}

pub fn import_package(
    connection: &PooledConnection,
    parts: Parts,
    user: &User,
) -> Result<(), ImportPackageError> {
    connection.transaction::<_, ImportPackageError, _>(|| {
        let repository = parts
            .texts
            .as_pairs()
            .iter()
            .find(|(key, _)| *key == "repository")
            .map(|(_, value)| *value)
            .ok_or_else(|| ImportPackageError::TextFieldNotFound(String::from("repository")))?;
        let repository = Repository::find_by_name(connection, repository)?
            .ok_or_else(|| ImportPackageError::RepositoryNotFound(repository.to_string()))?;
        let file = parts
            .files
            .into_inner()
            .into_iter()
            .filter(|(key, _)| key == "file")
            .map(|(_, result)| result)
            .next()
            .ok_or(ImportPackageError::FileNotFound)?
            .map_err(ImportPackageError::MultipartError)?
            .into_inner();
        let info = PackageInfo::from_file(file.path())?;
        create_or_update_package(&connection, &repository, user, &info)?;
        let package_path = path::package_file(
            &repository.name,
            &info.architecture,
            &info.name,
            &info.extension,
        );
        persist_package(file, &package_path)?;
        let repository_path =
            path::repository_file(&repository.name, &info.architecture, &repository.extension);
        add_package_to_repository(&package_path, &repository_path)?;
        Ok(())
    })
}

pub fn persist_package(file: NamedTempFile, path: &Path) -> Result<(), ImportPackageError> {
    let (_, file_path) = file.keep()?;
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    fs::copy(file_path, path)?;
    Ok(())
}

pub fn remove_package_from_repository(
    package: &str,
    repository_path: &std::path::Path,
) -> WarehouseResult {
    Command::new("repo-remove")
        .arg(repository_path)
        .arg(package)
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()?;
    Ok(())
}

pub fn update_package(
    connection: &PooledConnection,
    user: &User,
    info: &PackageInfo,
    package: &mut Package,
) -> Result<(), ImportPackageError> {
    let now = Utc::now().naive_utc();
    package.version = info.version.clone();
    package.description = info.description.clone();
    package.url = info.url.clone();
    package.build_date = info.build_date;
    package.compressed_size = info.compressed_size;
    package.installed_size = info.installed_size;
    package.license = info.licenses.join(" ");
    package.extension = info.extension.clone();
    package.maintainer_id = user.id.clone();
    package.modification_date = now;
    package.update(connection)?;
    package.delete_dependencies(connection)?;
    for dependency in &info.dependencies {
        let dependency = Dependency {
            id: Uuid::new_v4().to_string(),
            name: dependency.clone(),
            package_id: package.id.clone(),
        };
        dependency.create(connection)?;
    }
    package.delete_files(connection)?;
    for (filename, size) in &info.files {
        let file = File {
            id: Uuid::new_v4().to_string(),
            name: filename.clone(),
            size: *size,
            package_id: package.id.clone(),
        };
        file.create(connection)?;
    }
    let version = Version {
        id: Uuid::new_v4().to_string(),
        creation_date: now,
        version: info.version.clone(),
        maintainer_id: user.id.clone(),
        package_id: package.id.clone(),
    };
    version.create(connection)?;
    Ok(())
}
