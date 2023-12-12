use anyhow::{anyhow, Ok, Result};
use chrono::{
    format::{strftime::StrftimeItems, Item},
    NaiveDateTime,
};
use clap::{Parser, ValueHint};
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

#[derive(Parser, Debug)]
#[command(version, about, next_line_help = true)]
struct Cli {
    /// Directory containing images to be renamed
    #[arg(value_parser=parse_directory, value_hint=ValueHint::DirPath)]
    dir: PathBuf,

    /// Formatting syntax to use for new image filenames
    #[arg(short, long, value_parser=parse_format, default_value="%Y-%m-%d_%H-%M-%S")]
    format: Option<String>,
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

#[derive(Debug)]
enum Errors {
    NotADirectory,
    NoExifDateTimeAvailable,
    FileExists(PathBuf),
    InvalidDateTimeFormat,
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
            Self::InvalidDateTimeFormat => {
                write!(
                    f,
                    "Invalid date time format, use strftime formatting syntax"
                )
            }
        }
    }
}

impl error::Error for Errors {}

fn main() -> Result<()> {
    init_logger()?;
    traverse_dir()?;
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
fn traverse_dir() -> Result<()> {
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
    let cli = Cli::parse();
    let format = cli.format.as_deref().unwrap();

    path.file_stem()
        .and_then(|filename| filename.to_str())
        .and_then(|filename| NaiveDateTime::parse_from_str(filename, format).ok())
        .map_or(false, |_| true)
}

/// Generates the new filename for renaming a file.
/// The pattern is: YYYY-MM-DD_H-M-S.extension
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
