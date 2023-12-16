use std::{error, fmt, path::PathBuf};

#[derive(Debug)]
pub enum Errors {
    NotADirectory,
    NoExifDateTimeAvailable,
    FileExists(PathBuf),
    InvalidDateTimeFormat,
    NoParentFolder,
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
            Self::NoParentFolder => {
                write!(f, "Parent folder is not available")
            }
        }
    }
}

impl error::Error for Errors {}
