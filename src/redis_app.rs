use std::{collections::HashMap, error::Error, sync::Mutex, time::Duration, u64};

use crate::{
    rdb_file::RdbFile,
    redis_types::{Command, StreamEntry, StreamKey, ValueContainer},
    resp_serializer::{
        self, null_resp_string, to_err_string, to_resp_array, to_resp_bulk, to_resp_string,
    },
    utils,
};

#[derive(Debug)]
struct EntryValue {
    value: ValueContainer,
    expires_at: Option<u128>,
}

impl EntryValue {
    pub fn get_value(&self) -> Option<ValueContainer> {
        if let Some(exp) = self.expires_at {
            let current_time = utils::get_current_time_ms();
            if current_time < exp {
                return Some(self.value.clone());
            }
            return None;
        }
        return Some(self.value.clone());
    }
}

#[derive(Debug)]
pub struct RedisApp {
    memory: Mutex<HashMap<String, EntryValue>>,
    configurations: Mutex<HashMap<String, String>>,
}

impl RedisApp {
    pub fn new(args: impl Iterator<Item = String>) -> Self {
        let configurations = Mutex::new(Self::load_configs_from_args(args));

        let mut instance = RedisApp {
            memory: Mutex::new(HashMap::new()),
            configurations,
        };

        instance.load_from_rdb();

        println!("Instance initialized: {:?}", instance);

        instance
    }

    fn load_from_rdb(&mut self) {
        let config = self
            .configurations
            .lock()
            .expect("Failed to lock configurations hashmap");

        if let (Some(path), Some(name)) = (config.get("dir"), config.get("dbfilename")) {
            let mut path = path.to_owned();

            path.push('/');
            path.push_str(&name);
            println!("Looking for rdb at: {}", path);

            match RdbFile::open(path) {
                Ok(rdb) => {
                    let mut mem = self.memory.lock().expect("Failed to unlock memory hashmap");

                    for (key, (value, expires)) in rdb.memory {
                        let entry = EntryValue {
                            value: ValueContainer::String(value),
                            expires_at: expires,
                        };

                        _ = mem.insert(key, entry);
                    }
                }
                Err(err) => {
                    println!("Failed to load rdb: {:?}", err)
                }
            }
        } else {
            println!("RDB file path not provided");
        }
    }

    fn load_configs_from_args(mut args: impl Iterator<Item = String>) -> HashMap<String, String> {
        let mut configs = HashMap::new();

        while let Some(arg) = args.next() {
            match arg.as_str() {
                "--dir" => {
                    if let Some(dir_value) = args.next() {
                        _ = configs.insert("dir".to_owned(), dir_value);
                    }
                }
                "--dbfilename" => {
                    if let Some(filename_value) = args.next() {
                        _ = configs.insert("dbfilename".to_owned(), filename_value);
                    }
                }
                _ => {}
            }
        }
        println!("Configs: {:?}", configs);
        configs
    }

    fn set_command(
        &self,
        key: String,
        value: ValueContainer,
        expires_at: Option<u128>,
    ) -> Result<String, Box<dyn std::error::Error>> {
        let mut mem = self.memory.lock().expect("Failed to lock memory hashmap");

        let expires: Option<u128> = match expires_at {
            Some(ex) => Some(utils::get_current_time_ms() + ex),
            None => None,
        };

        let entry = EntryValue {
            value,
            expires_at: expires,
        };

        _ = mem.insert(key, entry);
        Ok(to_resp_string("OK".to_owned()))
    }

    fn get_command(&self, key: String) -> String {
        let mem = self.memory.lock().expect("Failed to lock memory hashmap");

        if let Some(entry) = mem.get(&key) {
            if let Some(value) = entry.get_value() {
                return to_resp_string(value.into());
            }
        }

        return null_resp_string();
    }

    pub async fn execute_command(
        &self,
        cmd: Command,
    ) -> Result<String, Box<dyn std::error::Error>> {
        match cmd {
            Command::Ping => Ok(Self::ping_command()),
            Command::Echo(arg) => Ok(Self::echo_command(arg)),
            Command::Get(key) => Ok(self.get_command(key)),
            Command::Set(key, value, expires_at) => self.set_command(key, value, expires_at),
            Command::ConfigGet(cfg) => Ok(self.config_get_command(cfg)),
            Command::Keys(arg) => Ok(self.keys_command(arg)),
            Command::Type(tp) => Ok(self.type_command(tp)),
            Command::XAdd(key, id, fields) => self.xadd_command(key, id, fields),
            Command::XRange(key, start, end) => self.xrange_command(key, start, end),
            Command::XRead(block_time, stream_keys, id) => {
                self.xread(block_time, stream_keys, id).await
            }
        }
    }

    fn ping_command() -> String {
        to_resp_bulk("PONG".to_owned())
    }

    fn echo_command(arg: ValueContainer) -> String {
        to_resp_bulk(arg.into())
    }

    fn config_get_command(&self, arg: String) -> String {
        let config: std::sync::MutexGuard<HashMap<String, String>> = self
            .configurations
            .lock()
            .expect("Failed to lock configurations hashmap");

        if let Some(value) = config.get(&arg) {
            let values = vec![arg, value.to_owned()];
            return to_resp_array(values);
        }

        null_resp_string()
    }

    fn keys_command(&self, _arg: String) -> String {
        let mem = self.memory.lock().expect("Failed to lock memory hashmap");

        let keys: Vec<&String> = mem.keys().collect();
        let keys_owned: Vec<String> = keys.iter().map(|s| s.to_owned().to_owned()).collect();
        to_resp_array(keys_owned)
    }

    fn type_command(&self, key: String) -> String {
        let mem = self.memory.lock().expect("Failed to lock memory hashmap");

        if let Some(entry) = mem.get(&key) {
            if let Some(value) = entry.get_value() {
                return match value {
                    ValueContainer::Stream(..) => to_resp_string("stream".to_owned()),
                    ValueContainer::String(_) => to_resp_string("string".to_owned()),
                    ValueContainer::Array(..) => to_resp_string("list".to_owned()),
                    ValueContainer::Integer(_) => to_resp_string("integer".to_owned()),
                };
            }
        }

        return to_resp_string("none".to_owned());
    }

    fn xadd_command(
        &self,
        key: String,
        id: String,
        fields: Vec<(String, String)>,
    ) -> Result<String, Box<dyn Error>> {
        if id.as_str() == "0-0" {
            return Ok(to_err_string(
                "ERR The ID specified in XADD must be greater than 0-0".to_owned(),
            ));
        }

        let mut mem = self.memory.lock().expect("Failed to lock memory hashmap");

        let last_key = self.get_last_stream_key(&key, &mem);
        let stream_key = StreamKey::from_string(&id, &last_key, None)?;

        if let Some(last) = last_key {
            if stream_key <= last {
                return Ok(to_err_string(String::from("ERR The ID specified in XADD is equal or smaller than the target stream top item")));
            }
        }

        let new_entry = StreamEntry {
            id: stream_key.clone(),
            fields,
        };

        if let Some(entry) = mem.get_mut(&key) {
            if let ValueContainer::Stream(ref mut stream) = entry.value {
                stream.push(new_entry);
                return Ok(to_resp_bulk(stream_key.into()));
            }
        }

        mem.insert(
            key,
            EntryValue {
                expires_at: None,
                value: ValueContainer::Stream(vec![new_entry]),
            },
        );

        return Ok(to_resp_bulk(stream_key.into()));
    }

    fn get_last_stream_key(
        &self,
        stream_id: &str,
        mem: &std::sync::MutexGuard<HashMap<String, EntryValue>>,
    ) -> Option<StreamKey> {
        let entry = mem.get(stream_id)?;

        if let ValueContainer::Stream(stream) = &entry.value {
            let last = stream.last()?;
            Some(last.id.clone())
        } else {
            None
        }
    }

    fn xrange_command(
        &self,
        key: String,
        start: String,
        end: String,
    ) -> Result<String, Box<dyn Error>> {
        let mem = self.memory.lock().expect("Failed to lock mem");
        let start_id = StreamKey::from_string(&start, &None, Some(0))?;
        let end_id = StreamKey::from_string(&end, &None, Some(u64::MAX))?;

        if end_id < start_id {
            return Ok(to_err_string(String::from("ERR Invalid range")));
        }

        if let Some(entry_value) = mem.get(&key) {
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

    async fn xread(
        &self,
        block_time: Option<u64>,
        stream_keys: Vec<String>,
        id: String,
    ) -> Result<String, Box<dyn Error>> {
        if let Some(block_time) = block_time {
            tokio::time::sleep(Duration::from_millis(block_time)).await;
        }

        let start_id = StreamKey::from_string(&id, &None, Some(0))?;
        let mem = self.memory.lock().expect("Failed to lock mem");

        let mut entry_parsed = Vec::new();

        for key in stream_keys {
            if let Some(entry) = mem.get(&key) {
                if let ValueContainer::Stream(stream) = &entry.value {
                    let idx_start = match stream.binary_search_by(|val| val.id.cmp(&start_id)) {
                        Ok(idx) => idx + 1,
                        Err(idx) => idx,
                    };

                    if idx_start >= stream.len() {
                        continue;
                    }

                    let slice = &stream[idx_start..];
                    let serialized = resp_serializer::slc_objects_to_resp(slice);
                    let name_serialized = resp_serializer::to_resp_bulk(key);
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
