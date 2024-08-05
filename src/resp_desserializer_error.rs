use std::{error::Error, fmt};

#[derive(Debug, Clone)]
pub struct RespDesserializerError {
    message: String,
}

impl RespDesserializerError {
    // Constructor to create a new error with a message
    pub fn new(msg: &str) -> RespDesserializerError {
        RespDesserializerError {
            message: msg.to_string(),
        }
    }
}

impl fmt::Display for RespDesserializerError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "RespDesserializerError: {}", self.message)
    }
}

impl Error for RespDesserializerError {}
