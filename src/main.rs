mod cli;
mod error;

use anyhow::{anyhow, Ok, Result};
use chrono::NaiveDateTime;
use clap::Parser;
use cli::Cli;
use error::Errors;
use exif::{Exif, Field, In, Reader, Tag};
use indicatif::{ProgressBar, ProgressStyle};
use log::{error, LevelFilter};
use simplelog::{ColorChoice, Config, TermLogger, TerminalMode};
use std::{
    fs::{self, File},
    io::BufReader,
    path::{Path, PathBuf},
};
use walkdir::WalkDir;

fn main() -> Result<()> {
    init()?;
    run()?;
    Ok(())
}

/// Init global log
fn init() -> Result<()> {
    TermLogger::init(
        LevelFilter::Error,
        Config::default(),
        TerminalMode::Mixed,
        ColorChoice::Auto,
    )?;

    Ok(())
}

/// Recursively read directory supplied in CLI argument and rename image files.
fn run() -> Result<()> {
    let cli = Cli::parse();
    let format = cli.format.as_deref().unwrap();
    println!("Renaming images using format `{}`:", format);
    println!();

    let dir = cli.dir;
    let entries = WalkDir::new(dir)
        .sort_by_file_name()
        .into_iter()
        .filter_map(|res| res.ok())
        .filter(|entry| entry.path().is_file())
        .collect::<Vec<_>>();

    let pb = get_progressbar(entries.len());

    for entry in entries {
        let path = entry.path();
        pb.set_message(path.display().to_string());

        if let Err(err) = rename(path) {
            pb.suspend(|| error!("{} - {}", path.display(), err));
        }

        pb.inc(1);
    }

    pb.println("");
    pb.finish_with_message("Finished renaming images");

    Ok(())
}

/// ProgressBar configuration and setup
fn get_progressbar(len: usize) -> ProgressBar {
    let pb = ProgressBar::new(len as u64);
    pb.set_style(
        ProgressStyle::with_template(
            "[{elapsed_precise}] [{wide_bar}] {human_pos}/{human_len:3} {msg}",
        )
        .unwrap()
        .progress_chars("#>-"),
    );

    pb
}

/// Rename a file to it's new filename.
/// Files that already adhere to target filename pattern are skipped.
/// If target filename already exists an error message is shown. This makes sure we don't accidentally overwrite images
fn rename(path: &Path) -> Result<()> {
    let file = File::open(path)?;
    let mut bufreader = BufReader::new(&file);
    let exif = Reader::new().read_from_container(&mut bufreader)?;
    let new_filename = filename(path, exif)?;

    if has_correct_filename(path, new_filename.as_str()) {
        return Ok(());
    }

    if let Some(parent) = path.parent() {
        let mut to = PathBuf::from(parent);
        to.push(new_filename);

        if !to.exists() {
            fs::rename(path, &to)?;
        } else {
            return Err(anyhow!(Errors::FileExists(to)));
        }
    }

    Ok(())
}

/// Check if file name already has correct format
fn has_correct_filename(path: &Path, filename: &str) -> bool {
    path.file_name()
        .and_then(|current_filename| current_filename.to_str())
        .map_or_else(|| false, |current_filename| current_filename == filename)
}

/// Generates the new filename
fn filename(path: &Path, exif: Exif) -> Result<String> {
    let exif_dt = get_datetime_from_exif(&exif)?.display_value().to_string();

    let dt = NaiveDateTime::parse_from_str(&exif_dt, "%Y-%m-%d %H:%M:%S")?;

    let cli = Cli::parse();
    let format = cli.format.as_deref().unwrap();
    let filename = dt.format(format);

    Ok(
        match path.extension().and_then(|extension| extension.to_str()) {
            Some(extension) => format!("{filename}.{}", extension.to_lowercase()),
            None => filename.to_string(),
        },
    )
}

/// Tries to get a date time value from EXIF data.
/// Checks different EXIF fields, if none of them exist returns an error.
fn get_datetime_from_exif(exif: &Exif) -> Result<&Field> {
    exif.get_field(Tag::DateTimeOriginal, In::PRIMARY)
        .or_else(|| exif.get_field(Tag::DateTimeDigitized, In::PRIMARY))
        .or_else(|| exif.get_field(Tag::DateTime, In::PRIMARY))
        .ok_or_else(|| anyhow!(Errors::NoExifDateTimeAvailable))
}
