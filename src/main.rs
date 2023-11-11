use anyhow::{anyhow, bail, Context, Result};
use chrono::NaiveDateTime;
use clap::Parser;
use exif::{Exif, In, Reader, Tag};
use std::{
    fs::{DirEntry, File},
    io::BufReader,
    path::PathBuf,
};

#[derive(Parser)]
struct Cli {
    dir: PathBuf,
}

#[derive(Debug)]
enum Errors {
    NotADirectory,
    NoExifDateTimeAvailable,
}

impl std::fmt::Display for Errors {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::NotADirectory => write!(f, "Not a directory"),
            Self::NoExifDateTimeAvailable => write!(f, "EXIF data has no date time parameter set"),
        }
    }
}

impl std::error::Error for Errors {}

fn traverse_dir(dir: &PathBuf) -> Result<()> {
    if !dir.is_dir() {
        bail!(Errors::NotADirectory);
    }

    for entry in dir
        .read_dir()
        .with_context(|| format!("Could not read directory `{}`", dir.display()))?
    {
        let e = entry?;
        if e.path().is_dir() {
            traverse_dir(&e.path())?;
        } else {
            rename(&e)?;
        }
    }

    Ok(())
}

fn rename(entry: &DirEntry) -> Result<()> {
    let file = File::open(entry.path())
        .with_context(|| format!("Could not open file `{}`", entry.path().display()))?;
    let mut bufreader = BufReader::new(&file);
    let exif = Reader::new().read_from_container(&mut bufreader);

    match exif {
        Ok(exif) => {
            let filename = filename(entry, exif);

            match filename {
                Ok(filename) => {
                    if let Some(parent) = entry.path().parent() {
                        let mut to = PathBuf::from(parent);
                        to.push(filename);

                        if entry.path() != to {
                            std::fs::rename(entry.path(), &to).with_context(|| {
                                format!("Could not rename `{}`", entry.path().display())
                            })?;
                            println!("{} -> {}", entry.path().display(), to.display());
                        }
                    }
                }
                Err(err) => {
                    eprintln!("{} -> {}", entry.path().display(), err)
                }
            }
        }
        Err(err) => eprintln!("{} -> {}", entry.path().display(), err),
    }

    Ok(())
}

fn filename(entry: &DirEntry, exif: Exif) -> Result<String> {
    let exif_dt = get_datetime_from_exif(exif)?;

    let dt = NaiveDateTime::parse_from_str(&exif_dt, "%Y-%m-%d %H:%M:%S")
        .with_context(|| format!("Unable to parse EXIF date"))?;

    let mut filename = dt.format("%Y-%m-%d_%H-%M-%S").to_string();
    let path = entry.path();
    let extension = path.extension();

    match extension {
        Some(extension) => {
            if let Some(ext) = extension.to_str() {
                filename.push('.');
                filename.push_str(&ext.to_lowercase());
            }
            Ok(filename)
        }
        None => Ok(filename),
    }
}

fn get_datetime_from_exif(exif: Exif) -> Result<String> {
    if let Some(datetime) = exif.get_field(Tag::DateTimeOriginal, In::PRIMARY) {
        return Ok(datetime.display_value().to_string());
    };

    if let Some(datetime) = exif.get_field(Tag::DateTimeDigitized, In::PRIMARY) {
        return Ok(datetime.display_value().to_string());
    };

    if let Some(datetime) = exif.get_field(Tag::DateTime, In::PRIMARY) {
        return Ok(datetime.display_value().to_string());
    };

    Err(anyhow!(Errors::NoExifDateTimeAvailable))
}

fn main() -> Result<()> {
    let args = Cli::parse();
    traverse_dir(&args.dir)?;
    Ok(())
}
