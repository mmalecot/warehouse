use std::path::PathBuf;

#[cfg(feature = "run_in_place")]
pub fn config_dir() -> PathBuf {
    PathBuf::from("config")
}

#[cfg(not(feature = "run_in_place"))]
pub fn config_dir() -> PathBuf {
    let mut path = PathBuf::new();
    path.push("/etc");
    path.push(env!("CARGO_PKG_NAME"));
    path
}

pub fn config_file() -> PathBuf {
    let mut path = config_dir();
    path.push("application");
    path.set_extension("toml");
    path
}

#[cfg(feature = "run_in_place")]
pub fn data_dir() -> PathBuf {
    PathBuf::from("data")
}

#[cfg(not(feature = "run_in_place"))]
pub fn data_dir() -> PathBuf {
    let mut path = PathBuf::new();
    path.push("/var");
    path.push("lib");
    path.push(env!("CARGO_PKG_NAME"));
    path
}

#[cfg(feature = "run_in_place")]
pub fn log_dir() -> PathBuf {
    PathBuf::from("log")
}

#[cfg(not(feature = "run_in_place"))]
pub fn log_dir() -> PathBuf {
    let mut path = PathBuf::new();
    path.push("/var");
    path.push("log");
    path.push(env!("CARGO_PKG_NAME"));
    path
}

pub fn log_file() -> PathBuf {
    let mut path = log_dir();
    path.push(env!("CARGO_PKG_NAME"));
    path.set_extension("log");
    path
}

pub fn package_file(repository: &str, architecture: &str, name: &str, extension: &str) -> PathBuf {
    let mut path = data_dir();
    path.push("packages");
    path.push(repository);
    path.push(architecture);
    path.push(name);
    path.set_extension(extension);
    path
}

pub fn repository_file(repository: &str, architecture: &str, extension: &str) -> PathBuf {
    let mut path = data_dir();
    path.push("packages");
    path.push(repository);
    path.push(architecture);
    path.push("repository");
    path.set_extension(extension);
    path
}

#[cfg(feature = "run_in_place")]
pub fn resources_dir() -> PathBuf {
    PathBuf::from("resources")
}

#[cfg(not(feature = "run_in_place"))]
pub fn resources_dir() -> PathBuf {
    let mut path = PathBuf::new();
    path.push("/usr");
    path.push("share");
    path.push(env!("CARGO_PKG_NAME"));
    path
}

pub fn static_files_dir() -> PathBuf {
    let mut path = resources_dir();
    path.push("static");
    path
}

pub fn templates_dir() -> PathBuf {
    let mut path = resources_dir();
    path.push("templates");
    path
}
