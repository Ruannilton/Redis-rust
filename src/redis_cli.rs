use std::{
    collections::HashMap,
    sync::Mutex,
    time::{SystemTime, UNIX_EPOCH},
};

use crate::parser::Command;

struct EntryValue {
    value: String,
    expires_at: Option<u128>,
}

pub struct RedisApp {
    memory: Mutex<HashMap<String, EntryValue>>,
}

impl RedisApp {
    pub fn new() -> Self {
        RedisApp {
            memory: Mutex::new(HashMap::new()),
        }
    }

    fn get_current_time_ms() -> u128 {
        let start = SystemTime::now();
        let since_the_epoch = start
            .duration_since(UNIX_EPOCH)
            .expect("Time went backwards");
        since_the_epoch.as_millis()
    }

    fn set_command(
        &self,
        key: String,
        value: String,
        expires_at: Option<u128>,
    ) -> Result<String, Box<dyn std::error::Error>> {
        let mut mem = self.memory.lock().unwrap();

        let expires: Option<u128> = match expires_at {
            Some(ex) => Some(Self::get_current_time_ms() + ex),
            None => None,
        };

        let entry = EntryValue {
            value,
            expires_at: expires,
        };

        _ = mem.insert(key, entry);
        Ok(Self::format_simple_string(String::from("OK")))
    }

    fn get_command(&self, key: String) -> String {
        let mem = self.memory.lock().unwrap();

        if let Some(entry) = mem.get(&key) {
            if let Some(expires_at) = entry.expires_at {
                let current_time = Self::get_current_time_ms();
                if current_time < expires_at {
                    return Self::format_simple_string(entry.value.to_owned());
                }
                return Self::format_null_bulk_string();
            }
            return Self::format_simple_string(entry.value.to_owned());
        }

        return Self::format_null_bulk_string();
    }

    pub fn execute_command(&self, cmd: Command) -> Result<String, Box<dyn std::error::Error>> {
        match cmd {
            Command::Ping => Ok(Self::ping_command()),
            Command::Echo(arg) => Ok(Self::echo_command(arg)),
            Command::Get(key) => Ok(self.get_command(key)),
            Command::Set(key, value, expires_at) => self.set_command(key, value, expires_at),
            _ => Ok(String::from("INVALID")),
        }
    }

    fn ping_command() -> String {
        let response = String::from("PONG");
        Self::format_bulk_string(response)
    }

    fn echo_command(arg: String) -> String {
        Self::format_bulk_string(arg)
    }

    fn format_bulk_string(arg: String) -> String {
        let encoded = format!("${}\r\n{}\r\n", arg.len(), arg);
        encoded
    }

    fn format_null_bulk_string() -> String {
        String::from("$-1\r\n")
    }

    fn format_simple_string(arg: String) -> String {
        let encoded = format!("+{}\r\n", arg);
        encoded
    }
}
