use std::{error::Error, fmt::Display};

#[derive(Debug)]
pub enum RespError {
    InvalidToken(String),
    InvalidUtf8Bytes,
    InvalidInteger,
    EmptyStream,
}

impl Error for RespError {}

impl Display for RespError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match &self {
            Self::EmptyStream => write!(f, "No data to parse"),
            Self::InvalidToken(tk) => write!(f, "Token {} not recognized", tk),
            Self::InvalidInteger => write!(f, "Failed to parse to integer"),
            Self::InvalidUtf8Bytes => write!(f, "Failed to parse to utf8 string"),
        }
    }
}
