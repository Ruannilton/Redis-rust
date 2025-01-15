use super::stream_entry::StreamEntry;

#[derive(Debug, Clone)]
pub enum ValueContainer {
    String(String),
    Stream(Vec<StreamEntry>),
    Integer(i64),
    Array(Vec<ValueContainer>),
    Boolean(bool),
    Null,
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

fn to_string(container: &ValueContainer) -> String {
    match container {
        ValueContainer::String(s) => s.to_owned(),
        ValueContainer::Integer(i) => i.to_string(),
        ValueContainer::Boolean(b) => b.to_string(),
        ValueContainer::Null => "null".to_owned(),
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
