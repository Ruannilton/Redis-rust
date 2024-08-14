use std::{error::Error, fmt::Display};

#[derive(Debug)]
pub enum RedisError {
    UnexpectedToken,
    InvalidCommand(String),
    NoTokenAvailable,
    InvalidArgument,
    LockError,
    InvalidStreamEntryId(String),
    RestoreRDBError,
    RDBDecodeSizeError,
    RDBInvalidHeader,
    IOError(std::io::Error),
    ParsingError,
    InvalidOpCode,
}

impl Error for RedisError {}

impl Display for RedisError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match &self {
            RedisError::UnexpectedToken => write!(f, "Unexpected token"),
            RedisError::NoTokenAvailable => write!(f, "No token available"),
            RedisError::InvalidArgument => write!(f, "Invalid argument"),
            RedisError::InvalidCommand(cmd) => write!(f, "Invalid command: {}", cmd),
            RedisError::InvalidStreamEntryId(v) => {
                write!(f, "Value provided is not a valid stream entry id: {}", v)
            }
            RedisError::LockError => write!(f, "Failed to lock resource"),
            RedisError::RestoreRDBError => write!(f, "Failed to restore from RDB"),
            RedisError::RDBDecodeSizeError => {
                write!(f, "Failed to parse bytes do size encoded value")
            }
            RedisError::IOError(err) => err.fmt(f),
            RedisError::RDBInvalidHeader => write!(f, "RDB header is invalid"),
            RedisError::ParsingError => write!(f, "Parsing error"),
            RedisError::InvalidOpCode => write!(f, "Invalid Op Code"),
        }
    }
}
