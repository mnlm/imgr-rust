use crate::error::Errors;
use anyhow::{anyhow, Result};
use chrono::format::{strftime::StrftimeItems, Item};
use clap::{ArgAction::Count, Args, Parser, ValueHint};
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

    #[command(flatten)]
    verbosity: Verbosity,
}

#[derive(Debug, Args)]
struct Verbosity {
    /// Add multiple flags for more log output
    ///
    /// By default, info will be reported. Add `-v` for debug, `-vv` for trace logging
    #[arg(short, long, action=Count)]
    verbose: u8,

    /// Add multiple flags to reduce log output
    ///
    /// By default, info will be reported. Add `-q` for warn, `-qq` for error, `-qqq` for no output
    #[arg(short, long, action=Count)]
    quiet: u8,
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
    verbosity: Verbosity,
}

impl Context {
    pub fn get() -> Self {
        let cli = Cli::parse();
        Context {
            dir: cli.dir,
            format: cli.format.unwrap(),
            verbosity: cli.verbosity,
        }
    }

    pub fn log_level(&self) -> LevelFilter {
        let level = 3 - (self.verbosity.quiet as i8) + (self.verbosity.verbose as i8);

        match level {
            i8::MIN..=0 => LevelFilter::Off,
            1 => LevelFilter::Error,
            2 => LevelFilter::Warn,
            3 => LevelFilter::Info,
            4 => LevelFilter::Debug,
            _ => LevelFilter::Trace,
        }
    }
}
