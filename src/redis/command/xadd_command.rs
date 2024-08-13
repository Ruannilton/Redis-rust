use crate::{
    redis::{
        redis_app::RedisApp,
        redis_error::RedisError,
        types::{
            entry_value::EntryValue, stream_entry::StreamEntry, stream_key::StreamKey,
            value_container::ValueContainer,
        },
    },
    resp::resp_serializer::{to_err_string, to_resp_bulk},
};

use super::command_trait::Command;

pub struct XAddCommand {
    key: String,
    entry_id: String,
    entry_fields: Vec<(String, String)>,
}

impl XAddCommand {
    pub fn new(key: String, entry_id: String, entry_fields: Vec<(String, String)>) -> Self {
        Self {
            key,
            entry_id,
            entry_fields,
        }
    }
}

impl Command for XAddCommand {
    async fn execute(self, app: &RedisApp) -> Result<String, RedisError> {
        if self.entry_id.as_str() == "0-0" {
            return Ok(to_err_string(
                "ERR The ID specified in XADD must be greater than 0-0".to_owned(),
            ));
        }

        let mut mem = app.memory.lock().map_err(|_| RedisError::LockError)?;
        let last_key = app.get_last_stream_key(&self.key, &mem);
        let stream_key = StreamKey::from_string(&self.entry_id, &last_key, None)
            .map_err(|_| RedisError::InvalidStreamEntryId(self.entry_id))?;

        if let Some(last) = last_key {
            if stream_key <= last {
                return Ok(to_err_string(String::from("ERR The ID specified in XADD is equal or smaller than the target stream top item")));
            }
        }

        let new_entry = StreamEntry {
            id: stream_key.clone(),
            fields: self.entry_fields,
        };

        if let Some(entry) = mem.get_mut(&self.key) {
            if let ValueContainer::Stream(ref mut stream) = entry.value {
                stream.push(new_entry);
                return Ok(to_resp_bulk(stream_key.into()));
            }
        }

        mem.insert(
            self.key,
            EntryValue {
                expires_at: None,
                value: ValueContainer::Stream(vec![new_entry]),
            },
        );

        Ok(to_resp_bulk(stream_key.into()))
    }
}
