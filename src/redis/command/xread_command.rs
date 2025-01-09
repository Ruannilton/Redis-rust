use std::{collections::HashMap, time::Duration};

use tokio::sync::MutexGuard;

use crate::{
    redis::{
        redis_app::RedisApp,
        redis_error::RedisError,
        types::{entry_value::EntryValue, stream_key::StreamKey, value_container::ValueContainer},
    },
    resp::resp_serializer::{self, null_resp_string},
};

use super::command_trait::Command;

pub struct XReadCommand {
    block_time: Option<u64>,
    stream_keys: Vec<String>,
    stream_ids: Vec<String>,
}

impl Command for XReadCommand {
    async fn execute(&self, app: &RedisApp) -> Result<String, RedisError> {
        let ids = self.calculate_stream_start_ids(app).await?;

        match self.block_time {
            Some(block_time) => {
                if block_time > 0 {
                    tokio::time::sleep(Duration::from_millis(block_time)).await;
                    let mem = app.memory.lock().await;
                    self.xread_reader(&self.stream_keys, &ids, &mem)
                } else {
                    loop {
                        tokio::time::sleep(Duration::from_millis(1000)).await;
                        let mem = app.memory.lock().await;
                        let resp = self.xread_reader(&self.stream_keys, &ids, &mem)?;
                        if resp != null_resp_string() {
                            return Ok(resp);
                        }
                        println!("No entry found");
                    }
                }
            }
            None => {
                let mem = app.memory.lock().await;
                self.xread_reader(&self.stream_keys, &ids, &mem)
            }
        }
    }
}

impl XReadCommand {
    pub fn new(block_time: Option<u64>, stream_keys: Vec<String>, stream_ids: Vec<String>) -> Self {
        Self {
            block_time,
            stream_keys,
            stream_ids,
        }
    }

    async fn calculate_stream_start_ids(
        &self,
        app: &RedisApp,
    ) -> Result<Vec<StreamKey>, RedisError> {
        let mut ids = Vec::new();

        let mem = app.memory.lock().await;
        let key_id = self.stream_keys.iter().zip(self.stream_ids.iter());
        for (key, id) in key_id {
            if id == "$" {
                let last_id = app.get_last_stream_key(key, &mem);
                let start_id = StreamKey::from_string(&id, &last_id, Some(0))
                    .map_err(|_| RedisError::InvalidStreamEntryId(id.to_owned()))?;
                ids.push(start_id);
            } else {
                let start_id = StreamKey::from_string(&id, &None, Some(0))
                    .map_err(|_| RedisError::InvalidStreamEntryId(id.to_owned()))?;
                ids.push(start_id);
            }
        }

        Ok(ids)
    }

    fn xread_reader(
        &self,
        stream_keys: &Vec<String>,
        ids: &Vec<StreamKey>,
        mem: &MutexGuard<HashMap<String, EntryValue>>,
    ) -> Result<String, RedisError> {
        let stream_with_time = stream_keys.iter().zip(ids.iter());
        let mut entry_parsed = Vec::new();

        for (key, id) in stream_with_time {
            if let Some(entry) = mem.get(key) {
                if let ValueContainer::Stream(stream) = &entry.value {
                    let idx_start = match stream.binary_search_by(|val| val.id.cmp(id)) {
                        Ok(idx) => idx + 1,
                        Err(idx) => idx,
                    };

                    if idx_start >= stream.len() {
                        continue;
                    }

                    let slice = &stream[idx_start..];
                    let serialized = resp_serializer::slc_objects_to_resp(slice);
                    let name_serialized = resp_serializer::to_resp_bulk(key.to_owned());
                    let blob_serialized = format!("*{}\r\n{}{}", 2, name_serialized, serialized);

                    entry_parsed.push(blob_serialized);
                }
            }
        }

        if entry_parsed.is_empty() {
            return Ok(resp_serializer::null_resp_string());
        }

        let mut result = format!("*{}\r\n", entry_parsed.len());
        for entry in entry_parsed {
            result.push_str(&entry)
        }

        Ok(result)
    }
}
