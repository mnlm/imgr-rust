use crate::error::Errors;
use anyhow::{anyhow, Result};
use chrono::format::{strftime::StrftimeItems, Item};
use clap::{Parser, ValueHint};
use std::path::PathBuf;

#[derive(Parser, Debug)]
#[command(version, about, next_line_help = true)]
pub struct Cli {
    /// Directory containing images to be renamed
    #[arg(value_parser=parse_directory, value_hint=ValueHint::DirPath)]
    pub dir: PathBuf,

    /// Formatting syntax to use for new image filenames
    #[arg(short, long, value_parser=parse_format, default_value="%Y-%m-%d_%H-%M-%S")]
    pub format: Option<String>,
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
