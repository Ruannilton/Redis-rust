use crate::resp_type::RespToken;

#[derive(Debug)]
pub enum Command {
    Ping,
    Echo(ValueContainer),
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

impl Into<String> for &StreamEntry {
    fn into(self) -> String {
        let fields = self
            .fields
            .iter()
            .map(|i| format!("{}: {}", i.0, i.1))
            .collect::<Vec<String>>()
            .join(", ");
        format!("{{{} [{}]}}", self.id, fields)
    }
}

#[derive(Debug, Clone)]
pub enum ValueContainer {
    String(String),
    Stream(Vec<StreamEntry>),
    Integer(i64),
    Array(Vec<ValueContainer>),
}

impl Into<String> for ValueContainer {
    fn into(self) -> String {
        to_string(&self)
    }
}

impl Into<String> for &ValueContainer {
    fn into(self) -> String {
        to_string(self)
    }
}

impl From<RespToken> for ValueContainer {
    fn from(value: RespToken) -> Self {
        from_aux(&value)
    }
}

impl From<&RespToken> for ValueContainer {
    fn from(value: &RespToken) -> Self {
        from_aux(value)
    }
}

fn to_string(container: &ValueContainer) -> String {
    match container {
        ValueContainer::String(s) => s.to_owned(),
        ValueContainer::Integer(i) => i.to_string(),
        ValueContainer::Stream(a) => a
            .iter()
            .map(|x| x.into())
            .collect::<Vec<String>>()
            .join(", "),
        ValueContainer::Array(a) => a
            .iter()
            .map(|x| to_string(x))
            .collect::<Vec<String>>()
            .join(", "),
    }
}

fn from_aux(value: &RespToken) -> ValueContainer {
    match value {
        RespToken::String(s) => ValueContainer::String(s.to_owned()),
        RespToken::Integer(i) => ValueContainer::Integer(i.to_owned()),
        RespToken::Error(s) => ValueContainer::String(s.to_owned()),
        RespToken::Array(a) => ValueContainer::Array(a.iter().map(|x| from_aux(x)).collect()),
    }
}
