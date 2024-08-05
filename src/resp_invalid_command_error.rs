use std::{
    error::Error,
    fmt::{self},
};

#[derive(Debug, Clone)]
pub struct RespInvalidCommandError {
    message: String,
}

impl RespInvalidCommandError {
    // Constructor to create a new error with a message
    pub fn new(msg: &str) -> RespInvalidCommandError {
        RespInvalidCommandError {
            message: msg.to_string(),
        }
    }
}

impl fmt::Display for RespInvalidCommandError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "RespInvalidCommandError: {}", self.message)
    }
}

impl Error for RespInvalidCommandError {}
