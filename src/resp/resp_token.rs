use std::fmt;

#[derive(Debug, Clone)]
pub enum RespToken {
    String(String),
    Error(String),
    Integer(i64),
    Array(Vec<RespToken>),
}

impl fmt::Display for RespToken {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match &self {
            RespToken::String(v) => write!(f, "{}", v),
            RespToken::Error(v) => write!(f, "{}", v),
            RespToken::Integer(v) => write!(f, "{}", v),
            RespToken::Array(v) => write!(f, "{:?}", v),
        }
    }
}
