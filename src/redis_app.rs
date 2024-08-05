use std::{collections::HashMap, error::Error, sync::Mutex};

use crate::{
    output_parser::{null_resp_string, to_err_string, to_resp_array, to_resp_bulk, to_resp_string},
    rdb_file::RdbFile,
    redis_types::{Command, StreamEntry, StreamKey, ValueContainer},
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
                return value.into();
            }
        }

        return null_resp_string();
    }

    pub fn execute_command(&self, cmd: Command) -> Result<String, Box<dyn std::error::Error>> {
        match cmd {
            Command::Ping => Ok(Self::ping_command()),
            Command::Echo(arg) => Ok(Self::echo_command(arg)),
            Command::Get(key) => Ok(self.get_command(key)),
            Command::Set(key, value, expires_at) => self.set_command(key, value, expires_at),
            Command::ConfigGet(cfg) => Ok(self.config_get_command(cfg)),
            Command::Keys(arg) => Ok(self.keys_command(arg)),
            Command::Type(tp) => Ok(self.type_command(tp)),
            Command::XAdd(key, id, fields) => self.xadd_command(key, id, fields),
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
        let mut mem = self.memory.lock().expect("Failed to lock memory hashmap");

        let stream_key = StreamKey::from_string(&id)?;

        let new_entry = StreamEntry {
            id: stream_key,
            fields,
        };

        if let Some(entry) = mem.get_mut(&key) {
            if let ValueContainer::Stream(ref mut stream) = entry.value {
                if let Some(last_entry) = stream.last() {
                    if last_entry.id < new_entry.id {
                        stream.push(new_entry);
                        return Ok(to_resp_bulk(key));
                    }
                } else {
                    let min_id = StreamKey::new(0, 1);
                    if min_id < new_entry.id {
                        stream.push(new_entry);
                        return Ok(to_resp_bulk(id));
                    }
                }

                return Ok(to_err_string(String::from("ERR The ID specified in XADD is equal or smaller than the target stream top item")));
            }
        }

        mem.insert(
            key,
            EntryValue {
                expires_at: None,
                value: ValueContainer::Stream(vec![new_entry]),
            },
        );

        return Ok(to_resp_bulk(id));
    }
}
