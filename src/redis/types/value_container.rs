use crate::resp::resp_token::RespToken;

use super::stream_entry::StreamEntry;

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
