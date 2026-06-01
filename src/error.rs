use std::error::Error;
use std::fmt::{Display, Formatter};
use std::io;
use std::num::ParseIntError;
use std::path::PathBuf;

pub type AppResult<T> = Result<T, AppError>;

#[derive(Debug)]
pub enum AppError {
    Io {
        source: io::Error,
        path: Option<PathBuf>,
        action: &'static str,
    },
    ParseInt {
        source: ParseIntError,
        value: String,
        field: &'static str,
    },
    InvalidArgument(String),
    InvalidIndex(String),
    MissingIndex(PathBuf),
    InvalidDate(String),
    UnsupportedExportFormat(String),
}

impl AppError {
    pub fn io(source: io::Error, path: impl Into<Option<PathBuf>>, action: &'static str) -> Self {
        Self::Io {
            source,
            path: path.into(),
            action,
        }
    }

    pub fn invalid_arg(message: impl Into<String>) -> Self {
        Self::InvalidArgument(message.into())
    }
}

impl Display for AppError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            AppError::Io {
                source,
                path,
                action,
            } => {
                if let Some(path) = path {
                    write!(f, "failed to {action} '{}': {source}", path.display())
                } else {
                    write!(f, "failed to {action}: {source}")
                }
            }
            AppError::ParseInt {
                source,
                value,
                field,
            } => {
                write!(f, "failed to parse {field} from '{value}': {source}")
            }
            AppError::InvalidArgument(message) => write!(f, "invalid argument: {message}"),
            AppError::InvalidIndex(message) => write!(f, "invalid index: {message}"),
            AppError::MissingIndex(path) => write!(
                f,
                "index file '{}' does not exist; run `rust_finder scan <path>` first",
                path.display()
            ),
            AppError::InvalidDate(date) => {
                write!(f, "invalid date '{date}', expected YYYY-MM-DD")
            }
            AppError::UnsupportedExportFormat(format) => {
                write!(
                    f,
                    "unsupported export format '{format}', expected json or csv"
                )
            }
        }
    }
}

impl Error for AppError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            AppError::Io { source, .. } => Some(source),
            AppError::ParseInt { source, .. } => Some(source),
            AppError::InvalidArgument(_)
            | AppError::InvalidIndex(_)
            | AppError::MissingIndex(_)
            | AppError::InvalidDate(_)
            | AppError::UnsupportedExportFormat(_) => None,
        }
    }
}

impl From<io::Error> for AppError {
    fn from(source: io::Error) -> Self {
        AppError::io(source, None, "perform IO")
    }
}
