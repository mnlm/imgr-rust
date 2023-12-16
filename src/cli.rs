use crate::error::Errors;
use anyhow::{anyhow, Result};
use chrono::format::{strftime::StrftimeItems, Item};
use clap::{ArgAction::Count, Parser, ValueHint};
use log::LevelFilter;
use std::path::PathBuf;

#[derive(Parser, Debug)]
#[command(version, about, next_line_help = true)]
struct Cli {
    /// Directory containing images to be renamed
    #[arg(value_parser=parse_directory, value_hint=ValueHint::DirPath)]
    dir: PathBuf,

    /// Formatting syntax to use for new image filenames
    #[arg(short, long, value_parser=parse_format, default_value="%Y-%m-%d_%H-%M-%S")]
    format: Option<String>,

    /// Add multiple flags for more output
    ///
    /// By default, logging is off. Add `-v` for errors, `-vv` for warnings etc.
    #[arg(short, long, action=Count)]
    verbose: u8,
}

/// Parse `format` argument to validate if it complies with the strftime formatting syntax
fn parse_format(format: &str) -> Result<String> {
    let mut items = StrftimeItems::new(format);
    if items.any(|item| item == Item::Error) {
        return Err(anyhow!(Errors::InvalidDateTimeFormat));
    }
    Ok(format.to_string())
}

/// Parse `dir` argument to validate if the directory exists
fn parse_directory(dir: &str) -> Result<PathBuf> {
    let dir = PathBuf::from(dir);
    if !dir.is_dir() {
        return Err(anyhow!(Errors::NotADirectory));
    }
    Ok(dir)
}

/// Context holds all relevant cli arguments
#[derive(Debug)]
pub struct Context {
    pub dir: PathBuf,
    pub format: String,
    verbose: u8,
}

impl Context {
    pub fn get() -> Self {
        let cli = Cli::parse();
        Context {
            dir: cli.dir,
            format: cli.format.unwrap(),
            verbose: cli.verbose,
        }
    }

    pub fn log_level(&self) -> LevelFilter {
        match self.verbose {
            0 => LevelFilter::Off,
            1 => LevelFilter::Error,
            2 => LevelFilter::Warn,
            3 => LevelFilter::Info,
            4 => LevelFilter::Debug,
            _ => LevelFilter::Trace,
        }
    }
}
