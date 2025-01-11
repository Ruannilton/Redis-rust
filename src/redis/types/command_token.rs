use std::{
    collections::{HashMap, VecDeque},
    sync::Arc,
    time::Duration,
};

use tokio::sync::MutexGuard;

use crate::{
    redis::{redis_app::RedisApp, redis_error::RedisError},
    resp::resp_serializer,
    utils,
};

use super::{
    entry_value::EntryValue, stream_entry::StreamEntry, stream_key::StreamKey,
    transactions::ClientId, value_container::ValueContainer,
};

#[derive(Debug, Clone)]
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
    Multi,
    Exec,
    Discard,
    Info(String),
    ReplConf(Vec<String>),
    Psync(String, i64),
    PostPsync(String, i64),
}

impl CommandToken {
    pub async fn execute(&self, request_id: u64, app: Arc<RedisApp>) -> String {
        match self {
            CommandToken::Ping => resp_serializer::to_resp_bulk("PONG".to_owned()),
            CommandToken::Echo(val) => resp_serializer::to_resp_bulk(val.into()),
            CommandToken::Set(key, value, exp) => {
                execute_set_cmd(key, value, exp.to_owned(), app).await
            }
            CommandToken::Get(key) => execute_get_cmd(key, app).await,
            CommandToken::ConfigGet(key) => execute_cnf_get(key, app).await,
            CommandToken::Keys(keys) => execute_keys_cmd(keys, app).await,
            CommandToken::Type(key) => execute_type_cmd(key, app).await,
            CommandToken::XAdd(key, entry_id, entry_fields) => {
                execute_xadd_cmd(key, entry_id, entry_fields, app).await
            }
            CommandToken::XRange(key, start, end) => execute_xrange_cmd(key, start, end, app).await,
            CommandToken::XRead(block_time, stream_keys, stream_ids) => {
                execute_xread_cmd(block_time, stream_keys, stream_ids, app).await
            }
            CommandToken::Inc(key) => execute_inc_cmd(key, app).await,
            CommandToken::Discard => exec_discard_cmd(request_id, app).await,
            CommandToken::Info(key) => exec_info_cmd(key, app).await,
            CommandToken::ReplConf(args) => execute_replconf_cmd(args).await,
            CommandToken::Psync(replication_id, offset) => {
                execute_psync_cmd(replication_id, offset, app).await
            }
            CommandToken::Exec => execute_exec_cmd(request_id, app).await,
            CommandToken::Multi => execute_multi_cmd(request_id, app).await,
            CommandToken::PostPsync(replication_id, offset) => {
                execute_post_psync_cmd(replication_id, offset, app).await
            }
        }
    }
}

async fn execute_post_psync_cmd(
    _replication_id: &str,
    _offset: &i64,
    _app: Arc<RedisApp>,
) -> String {
    let base64_file = "UkVESVMwMDEx+glyZWRpcy12ZXIFNy4yLjD6CnJlZGlzLWJpdHPAQPoFY3RpbWXCbQi8ZfoIdXNlZC1tZW3CsMQQAPoIYW9mLWJhc2XAAP/wbjv+wP9aog==";
    let bin_file = utils::base64_to_bin(base64_file).unwrap();
    let mut response: String = format!("${}\r\n", bin_file.len());
    response.push_str(&String::from_utf8_lossy(&bin_file));

    response
}

async fn execute_multi_cmd(client_id: u64, app: Arc<RedisApp>) -> String {
    let mut transactions = app.transactions.lock().await;
    if !transactions.contains_key(&client_id) {
        let transaction = VecDeque::new();
        transactions.insert(client_id, transaction);
    }
    resp_serializer::to_resp_string("OK".to_owned())
}

async fn execute_exec_cmd(request_id: ClientId, app: Arc<RedisApp>) -> String {
    let mut transactions = app.transactions.lock().await;

    match transactions.get(&request_id) {
        Some(commands) => {
            let mut responses = Vec::new();
            for cmd in commands {
                let resp = Box::pin(cmd.execute(request_id, app.clone())).await;
                responses.push(resp);
            }
            let mut response = format!("*{}\r\n", responses.len());
            for r in responses {
                response.push_str(&r);
            }
            transactions.remove(&request_id);
            return response;
        }
        None => return resp_serializer::to_err_string("ERR EXEC without MULTI".to_owned()),
    }
}

async fn execute_psync_cmd(_replication_id: &str, _offset: &i64, app: Arc<RedisApp>) -> String {
    let def = String::new();
    let replid = app.settings.master_replid.as_ref().unwrap_or(&def);
    let response: String = format!("FULLRESYNC {} {}", replid, 0);
    let serialized = resp_serializer::to_resp_string(response);
    serialized
}

async fn execute_replconf_cmd(args: &[String]) -> String {
    let first = args.get(0);
    let second = args.get(1);

    if let (Some(conf_name), Some(_conf_value)) = (first, second) {
        let name = conf_name.as_str();
        match name {
            "listening-port" => {}
            _ => {}
        }
    }

    let response = resp_serializer::to_resp_string("OK".into());
    response
}

async fn exec_info_cmd(_key: &str, app: Arc<RedisApp>) -> String {
    let mut response_str = String::new();

    response_str.push_str("# Replication\n");

    let instance_type = match app.settings.instance_type {
        crate::redis::types::instance_type::InstanceType::Master => "role:master",
        crate::redis::types::instance_type::InstanceType::Slave => "role:slave",
    };

    response_str.push_str(instance_type);

    if let Some(master_replid) = &app.settings.master_replid {
        let replid = format!("\nmaster_replid:{}", master_replid);
        response_str.push_str(replid.as_str());
    }

    let master_repl_offset = format!("\nmaster_repl_offset:{}", app.settings.master_repl_offset);
    response_str.push_str(master_repl_offset.as_str());

    let resp = resp_serializer::to_resp_bulk(response_str);

    resp
}

async fn exec_discard_cmd(request_id: u64, app: Arc<RedisApp>) -> String {
    let mut transactions = app.transactions.lock().await;

    match transactions.get(&request_id) {
        Some(_) => {
            transactions.remove(&request_id);
            return resp_serializer::to_resp_string("OK".to_owned());
        }
        None => return resp_serializer::to_err_string("ERR DISCARD without MULTI".to_owned()),
    }
}

async fn execute_inc_cmd(key: &str, app: Arc<RedisApp>) -> String {
    let mut mem = app.memory.lock().await;

    let new_val = match mem.get_mut(key) {
        Some(entry) => {
            let new_val = match &entry.value {
                ValueContainer::String(str) => match i64::from_str_radix(str, 10) {
                    Ok(v) => v + 1,
                    _ => {
                        return resp_serializer::to_err_string(
                            "ERR value is not an integer or out of range".into(),
                        )
                    }
                },
                ValueContainer::Integer(i) => *i + 1,
                _ => {
                    return resp_serializer::to_err_string(
                        "ERR value is not an integer or out of range".into(),
                    )
                }
            };

            entry.value = ValueContainer::Integer(new_val);
            new_val
        }
        None => {
            let entry = EntryValue {
                expires_at: None,
                value: ValueContainer::Integer(1),
            };
            mem.insert(key.to_owned(), entry);
            1
        }
    };

    resp_serializer::to_resp_integer(new_val)
}

async fn execute_xread_cmd(
    block_time: &Option<u64>,
    stream_keys: &[String],
    stream_ids: &[String],
    app: Arc<RedisApp>,
) -> String {
    let ids = calculate_stream_start_ids(stream_keys, stream_ids, app.clone())
        .await
        .unwrap();

    match block_time {
        Some(block_time) => {
            if *block_time > 0 {
                tokio::time::sleep(Duration::from_millis(*block_time)).await;
                let mem = app.memory.lock().await;
                xread_reader(stream_keys, &ids, &mem)
            } else {
                loop {
                    tokio::time::sleep(Duration::from_millis(1000)).await;
                    let mem = app.memory.lock().await;
                    let resp = xread_reader(stream_keys, &ids, &mem);
                    if resp != resp_serializer::null_resp_string() {
                        return resp;
                    }
                    println!("No entry found");
                }
            }
        }
        None => {
            let mem = app.memory.lock().await;
            xread_reader(stream_keys, &ids, &mem)
        }
    }
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

async fn execute_xrange_cmd(key: &str, start: &str, end: &str, app: Arc<RedisApp>) -> String {
    let mem = app.memory.lock().await;
    let start_id = StreamKey::from_string(&start.to_owned(), &None, Some(0))
        .map_err(|_| RedisError::InvalidStreamEntryId(start.to_owned()))
        .unwrap();
    let end_id = StreamKey::from_string(&end.to_owned(), &None, Some(u64::MAX))
        .map_err(|_| RedisError::InvalidStreamEntryId(end.to_owned()))
        .unwrap();

    if end_id < start_id {
        return resp_serializer::to_err_string(String::from("ERR Invalid range"));
    }

    if let Some(entry_value) = mem.get(key) {
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
            return serialized;
        }
    }

    resp_serializer::to_err_string(String::from("ERR The ID specified not exists"))
}

async fn execute_xadd_cmd(
    key: &str,
    entry_id: &str,
    entry_fields: &[(String, String)],
    app: Arc<RedisApp>,
) -> String {
    if entry_id == "0-0" {
        return resp_serializer::to_err_string(
            "ERR The ID specified in XADD must be greater than 0-0".to_owned(),
        );
    }

    let mut mem = app.memory.lock().await;
    let last_key = app.get_last_stream_key(key, &mem);
    let stream_key_result = StreamKey::from_string(&entry_id.to_owned(), &last_key, None);

    if stream_key_result.is_err() {
        return resp_serializer::to_err_string("INVALID_COMMAND".into());
    }

    let stream_key = stream_key_result.unwrap();

    if let Some(last) = last_key {
        if stream_key <= last {
            return resp_serializer::to_err_string(String::from(
                "ERR The ID specified in XADD is equal or smaller than the target stream top item",
            ));
        }
    }

    let new_entry = StreamEntry {
        id: stream_key.clone(),
        fields: entry_fields.to_owned(),
    };

    if let Some(entry) = mem.get_mut(key) {
        if let ValueContainer::Stream(ref mut stream) = entry.value {
            stream.push(new_entry);
            return resp_serializer::to_resp_bulk(stream_key.into());
        }
    }

    mem.insert(
        key.to_owned(),
        EntryValue {
            expires_at: None,
            value: ValueContainer::Stream(vec![new_entry]),
        },
    );

    resp_serializer::to_resp_bulk(stream_key.into())
}

async fn execute_type_cmd(key: &str, app: Arc<RedisApp>) -> String {
    let mem = app.memory.lock().await;

    if let Some(value) = mem.get(key).and_then(|entry| entry.get_value()) {
        match value {
            ValueContainer::Stream(..) => resp_serializer::to_resp_string("stream".to_owned()),
            ValueContainer::String(_) => resp_serializer::to_resp_string("string".to_owned()),
            ValueContainer::Array(..) => resp_serializer::to_resp_string("list".to_owned()),
            ValueContainer::Integer(_) => resp_serializer::to_resp_string("integer".to_owned()),
        }
    } else {
        resp_serializer::to_resp_string("none".to_owned())
    }
}

async fn execute_keys_cmd(_keys: &str, app: Arc<RedisApp>) -> String {
    let mem = app.memory.lock().await;

    let keys: Vec<&String> = mem.keys().collect();
    let keys_owned: Vec<String> = keys.iter().map(|s| s.to_owned().to_owned()).collect();
    resp_serializer::to_resp_array(keys_owned)
}

async fn execute_cnf_get(key: &str, app: Arc<RedisApp>) -> String {
    let configs = app.settings.to_hashmap();
    if let Some(value) = configs.get(key) {
        let value_arr = vec![key.to_owned(), value.to_owned()];
        resp_serializer::to_resp_array(value_arr)
    } else {
        resp_serializer::null_resp_string()
    }
}

async fn execute_set_cmd(
    key: &String,
    value: &ValueContainer,
    exp: Option<u128>,
    app: Arc<RedisApp>,
) -> String {
    let mut mem = app.memory.lock().await;

    let expires: Option<u128> = match exp {
        Some(ex) => Some(utils::get_current_time_ms() + ex),
        None => None,
    };

    let entry = EntryValue {
        value: value.to_owned(),
        expires_at: expires,
    };

    _ = mem.insert(key.to_owned(), entry);
    resp_serializer::to_resp_string("OK".to_owned())
}

async fn execute_get_cmd(key: &String, app: Arc<RedisApp>) -> String {
    let map = app.memory.lock().await;

    if let Some(value) = map.get(key).and_then(|f| f.get_value()) {
        let value_str: String = value.into();
        resp_serializer::to_resp_string(value_str)
    } else {
        resp_serializer::null_resp_string()
    }
}
