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
    XAdd(String, String, Vec<(String, String)>),
}

#[derive(Debug, Clone)]
pub struct StreamEntry {
    pub id: String,
    pub fields: Vec<(String, String)>,
}

#[derive(Debug, Clone)]
pub enum ValueContainer {
    String(String),
    Stream(Vec<StreamEntry>),
}

impl ValueContainer {
    pub fn to_resp_string(&self) -> String {
        match self {
            ValueContainer::String(value) => format!("+{}\r\n", value),
            ValueContainer::Stream(..) => format!("+\r\n"),
        }
    }
}
