use std::error::Error;
use std::fmt::Display;

#[derive(Debug)]
#[non_exhaustive]
pub enum JasonError {
    /// An error occurred while reading from or writing to the source.
    Io,
    /// The index was corrupt or out of bounds.
    Index,
    /// The key was invalid or not found.
    InvalidKey,
    /// The JSON value was invalid.
    JsonError,
    /// An unknown error occurred.
    Unknown,
}

impl Display for JasonError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl Error for JasonError {}
