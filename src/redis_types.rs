#[derive(Debug)]
pub enum Command {
    Ping,
    Echo(String),
    Invalid,
    Set(String, ValueContainer, Option<u128>),
    Get(String),
    ConfigGet(String),
    Keys(String),
    Type(String),
}

#[derive(Debug)]
pub enum ValueContainer {
    String(String),
}

impl ValueContainer {
    pub fn to_resp_string(&self) -> String {
        match self {
            ValueContainer::String(value) => format!("+{}\r\n", value),
        }
    }
}
