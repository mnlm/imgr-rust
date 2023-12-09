use anyhow::{anyhow, Context, Result};
use chrono::NaiveDateTime;
use clap::Parser;
use exif::{Exif, Field, In, Reader, Tag};
use std::{
    error, fmt,
    fs::{self, File},
    io::BufReader,
    path::{Path, PathBuf},
};

#[derive(Parser, Debug)]
struct Cli {
    dir: PathBuf,
}

#[derive(Debug)]
enum Errors {
    NotADirectory,
    NoExifDateTimeAvailable,
    FileExists,
}

impl fmt::Display for Errors {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::NotADirectory => write!(f, "Not a directory"),
            Self::NoExifDateTimeAvailable => write!(f, "EXIF data has no date time parameter set"),
            Self::FileExists => write!(f, "File already exists"),
        }
    }
}

impl error::Error for Errors {}

/// Recursively read directory supplied in CLI argument and rename image files.
fn traverse_dir(dir: &Path) -> Result<()> {
    if !dir.is_dir() {
        return Err(anyhow!(Errors::NotADirectory));
    }

    for entry in dir
        .read_dir()
        .with_context(|| format!("Could not read directory `{}`", dir.display()))?
    {
        let path = entry?.path();
        if path.is_dir() {
            traverse_dir(&path)?;
        } else if let Err(err) = rename(&path) {
            eprintln!("{} -> {}", path.display(), err);
        };
    }

    Ok(())
}

/// Rename a file to it's new filename.
/// If target filename already exists, nothing happens.
fn rename(path: &Path) -> Result<()> {
    let file =
        File::open(path).with_context(|| format!("Could not open file `{}`", path.display()))?;

    let mut bufreader = BufReader::new(&file);
    let exif = Reader::new().read_from_container(&mut bufreader)?;

    let filename = filename(path, exif)?;

    if let Some(parent) = path.parent() {
        let mut to = PathBuf::from(parent);
        to.push(filename);

        if !to.exists() {
            fs::rename(path, &to)
                .with_context(|| format!("Could not rename `{}`", path.display()))?;
            println!("{} -> {}", path.display(), to.display());
        } else {
            return Err(anyhow!(Errors::FileExists));
        }
    }

    Ok(())
}

/// Generates the new filename for renaming a file.
/// The pattern is: YYYY-MM-DD_H-M-S.extension
fn filename(path: &Path, exif: Exif) -> Result<String> {
    let exif_dt = get_datetime_from_exif(&exif)?.display_value().to_string();

    let dt = NaiveDateTime::parse_from_str(&exif_dt, "%Y-%m-%d %H:%M:%S")
        .with_context(|| "Unable to parse EXIF date".to_string())?;

    let filename = dt.format("%Y-%m-%d_%H-%M-%S");

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

fn main() -> Result<()> {
    let args = Cli::parse();
    traverse_dir(&args.dir)?;
    Ok(())
}
