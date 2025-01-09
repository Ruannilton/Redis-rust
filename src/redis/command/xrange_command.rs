use crate::{
    redis::{
        redis_app::RedisApp,
        redis_error::RedisError,
        types::{stream_key::StreamKey, value_container::ValueContainer},
    },
    resp::resp_serializer::{self, to_err_string},
};

use super::command_trait::Command;

pub struct XRangeCommand {
    key: String,
    start: String,
    end: String,
}

impl XRangeCommand {
    pub fn new(key: String, start: String, end: String) -> Self {
        Self { key, start, end }
    }
}

impl Command for XRangeCommand {
    async fn execute(&self, app: &RedisApp) -> Result<String, RedisError> {
        let mem = app.memory.lock().await;
        let start_id = StreamKey::from_string(&self.start, &None, Some(0))
            .map_err(|_| RedisError::InvalidStreamEntryId(self.start.to_owned()))?;
        let end_id = StreamKey::from_string(&self.end, &None, Some(u64::MAX))
            .map_err(|_| RedisError::InvalidStreamEntryId(self.end.to_owned()))?;

        if end_id < start_id {
            return Ok(to_err_string(String::from("ERR Invalid range")));
        }

        if let Some(entry_value) = mem.get(&self.key) {
            if let ValueContainer::Stream(stream) = &entry_value.value {
                let idx_start = match stream.binary_search_by(|val| val.id.cmp(&start_id)) {
                    Ok(idx) => idx,
                    Err(idx) => idx,
                };

                let idx_end = match stream.binary_search_by(|val| val.id.cmp(&end_id)) {
                    Ok(idx) => idx + 1,
                    Err(idx) => idx,
                };

                let slice = &stream[idx_start..idx_end];
                let serialized = resp_serializer::slc_objects_to_resp(slice);
                return Ok(serialized);
            }
        }

        Ok(to_err_string(String::from(
            "ERR The ID specified not exists",
        )))
    }
}
