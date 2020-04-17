use crate::{
    core::{config::LoggerConfig, error::SetupLoggerError},
    utils::path,
};
use chrono::Local;
use colored::Colorize;
use fern::{
    colors::{Color, ColoredLevelConfig},
    Dispatch,
};
use log::LevelFilter;
use std::{fs, io, thread};

pub fn setup(config: &LoggerConfig) -> Result<(), SetupLoggerError> {
    let mut dispatch = Dispatch::new()
        .level(LevelFilter::Off)
        .level_for("actix_web", config.level)
        .level_for(env!("CARGO_PKG_NAME"), config.level)
        .chain(setup_terminal_dispatch(&config));
    if config.file_dispatch {
        dispatch = dispatch.chain(setup_file_dispatch(&config)?);
    }
    dispatch.apply()?;
    Ok(())
}

fn setup_file_dispatch(config: &LoggerConfig) -> Result<Dispatch, SetupLoggerError> {
    let time_format = config.time_format.clone();
    Ok(Dispatch::new()
        .format(move |out, message, record| {
            out.finish(format_args!(
                "{date} | {thread:>20} | {level:5} | {target:30} | {message}",
                date = Local::now().format(&time_format),
                thread = thread::current().name().unwrap_or(""),
                level = record.level(),
                target = record.target(),
                message = message,
            ))
        })
        .chain(fern::log_file({
            let path = path::log_file();
            if let Some(parent) = path.parent() {
                fs::create_dir_all(parent)?;
            }
            path
        })?))
}

fn setup_terminal_dispatch(config: &LoggerConfig) -> Dispatch {
    let time_format = config.time_format.clone();
    Dispatch::new()
        .format(move |out, message, record| {
            out.finish(format_args!(
                "{date} | {thread:>20} | {level:5} | {target:30} | {message}",
                date = Local::now().format(&time_format),
                thread = thread::current().name().unwrap_or("").magenta(),
                level = ColoredLevelConfig::new()
                    .info(Color::Green)
                    .debug(Color::Blue)
                    .color(record.level()),
                target = record.target().cyan(),
                message = message,
            ))
        })
        .chain(io::stdout())
}
