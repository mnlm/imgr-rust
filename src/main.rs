use anyhow::{anyhow, Ok, Result};
use chrono::NaiveDateTime;
use clap::Parser;
use exif::{Exif, Field, In, Reader, Tag};
use indicatif::{ProgressBar, ProgressStyle};
use log::{error, LevelFilter};
use simplelog::{ColorChoice, Config, TermLogger, TerminalMode};
use std::{
    error, fmt,
    fs::{self, File},
    io::BufReader,
    path::{Path, PathBuf},
};
use walkdir::WalkDir;

const TARGET_FORMAT: &str = "%Y-%m-%d_%H-%M-%S";

#[derive(Parser, Debug)]
struct Cli {
    dir: PathBuf,
}

#[derive(Debug)]
enum Errors {
    NotADirectory,
    NoExifDateTimeAvailable,
    FileExists(PathBuf),
}

impl fmt::Display for Errors {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::NotADirectory => write!(f, "Not a directory"),
            Self::NoExifDateTimeAvailable => {
                write!(f, "EXIF data has no date time parameter set")
            }
            Self::FileExists(path) => {
                write!(f, "{} already exists", path.display())
            }
        }
    }
}

impl error::Error for Errors {}

fn main() -> Result<()> {
    init_logger()?;

    let args = Cli::parse();
    traverse_dir(&args.dir)?;
    Ok(())
}

// Init global log
fn init_logger() -> Result<()> {
    TermLogger::init(
        LevelFilter::Error,
        Config::default(),
        TerminalMode::Mixed,
        ColorChoice::Auto,
    )?;

    Ok(())
}

/// Recursively read directory supplied in CLI argument and rename image files.
fn traverse_dir(dir: &Path) -> Result<()> {
    if !dir.is_dir() {
        return Err(anyhow!(Errors::NotADirectory));
    }

    println!("Renaming images using format `{}`:", TARGET_FORMAT);
    println!();

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
/// If target filename already exists, nothing happens.
fn rename(path: &Path) -> Result<()> {
    if has_correct_filename(path) {
        return Ok(());
    }

    let file = File::open(path)?;
    let mut bufreader = BufReader::new(&file);
    let exif = Reader::new().read_from_container(&mut bufreader)?;
    let filename = filename(path, exif)?;

    if let Some(parent) = path.parent() {
        let mut to = PathBuf::from(parent);
        to.push(filename);

        if !to.exists() {
            fs::rename(path, &to)?;
        } else {
            return Err(anyhow!(Errors::FileExists(to)));
        }
    }

    Ok(())
}

/// Check if file name is the correct date time pattern
fn has_correct_filename(path: &Path) -> bool {
    path.file_stem()
        .and_then(|filename| filename.to_str())
        .and_then(|filename| NaiveDateTime::parse_from_str(filename, TARGET_FORMAT).ok())
        .map_or(false, |_| true)
}

/// Generates the new filename for renaming a file.
/// The pattern is: YYYY-MM-DD_H-M-S.extension
fn filename(path: &Path, exif: Exif) -> Result<String> {
    let exif_dt = get_datetime_from_exif(&exif)?.display_value().to_string();

    let dt = NaiveDateTime::parse_from_str(&exif_dt, "%Y-%m-%d %H:%M:%S")?;

    let filename = dt.format(TARGET_FORMAT);

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
    if let Some(datetime) = exif.get_field(Tag::DateTimeOriginal, In::PRIMARY) {
        return Ok(datetime);
    };

    if let Some(datetime) = exif.get_field(Tag::DateTimeDigitized, In::PRIMARY) {
        return Ok(datetime);
    };

    if let Some(datetime) = exif.get_field(Tag::DateTime, In::PRIMARY) {
        return Ok(datetime);
    };

    Err(anyhow!(Errors::NoExifDateTimeAvailable))
}
