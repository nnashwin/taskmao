extern crate clap;

use std;
use std::error::Error as StdError;
use std::fmt;
use std::io;

pub type Result<T> = std::result::Result<T, TError>;

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
    Task,
}

#[derive(Debug)]
pub struct TError {
    kind: ErrorKind, 
    err: Box<dyn StdError + Send + Sync>,
}

impl TError {
    pub fn exit (&self) {
        werr!("{}\n", self);
        std::process::exit(1);
    }

    pub fn new<E>(kind: ErrorKind, err: E) -> TError 
        where E: Into<Box<dyn StdError + Send + Sync>>
    {
        TError {
            err: err.into(),
            kind: kind,
        }   
    }
}

impl fmt::Display for TError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self.kind {
            ErrorKind::Io => write!(f, "{}", self.err),
            ErrorKind::Misc => write!(f, "{}", self.err),
            ErrorKind::Task => write!(f, "{}", self.err),
        }
    }
}

impl StdError for TError {
    fn description(&self) -> &str {
        match self.kind {
            ErrorKind::Io => "io error",
            ErrorKind::Misc => "misc error",
            ErrorKind::Task => "task error",
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
