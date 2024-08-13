use super::value_container::ValueContainer;

#[derive(Debug)]
pub enum CommandToken {
    Ping,
    Echo(ValueContainer),
    Set(String, ValueContainer, Option<u128>),
    Get(String),
    ConfigGet(String),
    Keys(String),
    Type(String),
    XAdd(String, String, Vec<(String, String)>),
    XRange(String, String, String),
    XRead(Option<u64>, Vec<String>, Vec<String>),
    Inc(String),
}
