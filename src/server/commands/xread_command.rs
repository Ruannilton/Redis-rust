use std::{collections::HashMap, sync::Arc, time::Duration};

use tokio::sync::MutexGuard;

use crate::{
    resp::resp_serializer,
    resp_desserializer::RespTk,
    server::redis_app::RedisApp,
    types::{
        entry_value::EntryValue, execution_response::ExecResponse, redis_error::RedisError,
        stream_key::StreamKey, value_container::ValueContainer,
    },
};

use super::command_utils::get_next_arg_string;

pub async fn execute_xread(app: Arc<RedisApp>, token: &RespTk) -> ExecResponse {
    let (block_time, stream_keys, stream_ids) = get_parameters(token);

    let ids = calculate_stream_start_ids(&stream_keys, &stream_ids, app.clone())
        .await
        .unwrap();

    match block_time {
        Some(block_time) => {
            if block_time > 0 {
                tokio::time::sleep(Duration::from_millis(block_time)).await;
                let mem = app.memory.lock().await;
                xread_reader(&stream_keys, &ids, &mem).into()
            } else {
                loop {
                    tokio::time::sleep(Duration::from_millis(1000)).await;
                    let mem = app.memory.lock().await;
                    let resp = xread_reader(&stream_keys, &ids, &mem);
                    if resp != resp_serializer::null_resp_string() {
                        return resp.into();
                    }
                }
            }
        }
        None => {
            let mem = app.memory.lock().await;
            xread_reader(&stream_keys, &ids, &mem).into()
        }
    }
}

fn get_parameters(token: &RespTk) -> (Option<u64>, Vec<String>, Vec<String>) {
    let mut args = token.get_command_args();
    let mut block_time: Option<u64> = None;
    let mut stream_names = Vec::new();
    while let Some(tk_content) = get_next_arg_string(&mut args) {
        match tk_content.to_uppercase().as_str() {
            "BLOCK" => {
                if let Some(time) = get_next_arg_string(&mut args) {
                    block_time = u64::from_str_radix(&time, 10).ok();
                }
            }
            "STREAMS" => {
                while let Some(stream_name) = get_next_arg_string(&mut args) {
                    stream_names.push(stream_name);
                }
            }
            _ => continue,
        }
    }
    let stream_ids = stream_names.split_off(stream_names.len() / 2);

    (block_time, stream_names, stream_ids)
}

async fn calculate_stream_start_ids(
    stream_keys: &[String],
    stream_ids: &[String],
    app: Arc<RedisApp>,
) -> Result<Vec<StreamKey>, RedisError> {
    let mut ids = Vec::new();

    let mem = app.memory.lock().await;
    let key_id = stream_keys.iter().zip(stream_ids.iter());
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
    stream_keys: &[String],
    ids: &Vec<StreamKey>,
    mem: &MutexGuard<HashMap<String, EntryValue>>,
) -> String {
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
        return resp_serializer::null_resp_string();
    }

    let mut result = format!("*{}\r\n", entry_parsed.len());
    for entry in entry_parsed {
        result.push_str(&entry)
    }

    result
}
