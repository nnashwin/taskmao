extern crate clap;

use std;
use std::error::Error as StdError;
use std::fmt;
use std::io;
use toml::de::Error as TomlError;

pub type TResult<T> = Result<T, TError>;

macro_rules! werr(
    ($($arg:tt)*) => ({
        use std::io::{Write, stderr};
        write!(&mut stderr(), $($arg)*).unwrap();
    })
);

#[derive(Debug)]
pub enum ErrorKind {
    Io,
    Misc,
    Sql,
}

#[derive(Debug)]
pub struct TError {
    kind: ErrorKind,
    err: Box<dyn StdError + Send + Sync>,
}

impl TError {
    pub fn exit(&self) {
        werr!("{}\n", self);
        std::process::exit(1);
    }

    pub fn new<E>(kind: ErrorKind, err: E) -> TError
    where
        E: Into<Box<dyn StdError + Send + Sync>>,
    {
        TError {
            err: err.into(),
            kind,
        }
    }
}

impl fmt::Display for TError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self.kind {
            ErrorKind::Io => write!(f, "{}", self.err),
            ErrorKind::Misc => write!(f, "{}", self.err),
            ErrorKind::Sql => write!(f, "{}", self.err),
        }
    }
}

impl StdError for TError {
    fn description(&self) -> &str {
        match self.kind {
            ErrorKind::Io => "io error",
            ErrorKind::Misc => "misc error",
            ErrorKind::Sql => "sql error",
        }
    }
}

impl From<io::Error> for TError {
    fn from(err: io::Error) -> TError {
        TError::new(ErrorKind::Io, err)
    }
}

impl From<clap::Error> for TError {
    fn from(err: clap::Error) -> TError {
        TError::new(ErrorKind::Misc, err)
    }
}

impl From<rusqlite::Error> for TError {
    fn from(err: rusqlite::Error) -> TError {
        TError::new(ErrorKind::Sql, err)
    }
}

impl From<chrono::ParseError> for TError {
    fn from(err: chrono::ParseError) -> TError {
        TError::new(ErrorKind::Misc, err)
    }
}

impl From<TomlError> for TError {
    fn from(err: TomlError) -> TError {
        TError::new(ErrorKind::Io, err)
    }
}
