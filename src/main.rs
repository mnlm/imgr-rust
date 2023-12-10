use anyhow::{anyhow, Ok, Result};
use chrono::NaiveDateTime;
use clap::Parser;
use exif::{Exif, Field, In, Reader, Tag};
use log::{error, info, LevelFilter};
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

fn init_logger() -> Result<()> {
    TermLogger::init(
        LevelFilter::Info,
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

    let entries = WalkDir::new(dir)
        .sort_by_file_name()
        .into_iter()
        .filter_map(|res| res.ok());

    for entry in entries {
        let path = entry.path();

        if path.is_file() {
            if let Err(err) = rename(path) {
                error!("{} - {}", path.display(), err.to_string());
            }
        }
    }

    Ok(())
}

/// Rename a file to it's new filename.
/// Files that already adhere to target filename pattern are skipped.
/// If target filename already exists, nothing happens.
fn rename(path: &Path) -> Result<()> {
    if has_correct_filename(path) {
        info!(
            "{} - {}",
            path.display(),
            "Already has the correct filename pattern"
        );

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

            info!("{} - {}", path.display(), to.display().to_string());
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
