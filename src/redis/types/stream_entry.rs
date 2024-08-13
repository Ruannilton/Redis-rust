use crate::resp::resp_serializer::{to_resp_array, to_resp_bulk, RespSerializer};

use super::stream_key::StreamKey;

#[derive(Debug, Clone)]
pub struct StreamEntry {
    pub id: StreamKey,
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
        let id_str: String = self.id.clone().into();
        format!("{{{} [{}]}}", id_str, fields)
    }
}

impl RespSerializer for StreamEntry {
    fn to_resp(&self) -> String {
        let fields_array: Vec<String> = self
            .fields
            .iter()
            .map(|x| [x.0.clone(), x.1.clone()])
            .flatten()
            .collect();
        let fields_resp = to_resp_array(fields_array);
        let id_resp = to_resp_bulk(self.id.into());
        format!("*2\r\n{id_resp}{fields_resp}")
    }
}
