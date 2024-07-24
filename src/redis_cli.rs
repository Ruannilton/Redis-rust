use std::{collections::HashMap, sync::Mutex};

use crate::parser::Command;

pub struct RedisApp {
    memory: Mutex<HashMap<String, String>>,
}

impl RedisApp {
    pub fn new() -> Self {
        RedisApp {
            memory: Mutex::new(HashMap::new()),
        }
    }

    fn set_command(
        &self,
        key: String,
        value: String,
    ) -> Result<String, Box<dyn std::error::Error>> {
        let mut mem = self.memory.lock().unwrap();
        _ = mem.insert(key, value);
        Ok(Self::format_simple_string(String::from("OK")))
    }

    fn get_command(&self, key: String) -> Result<String, Box<dyn std::error::Error>> {
        let mem = self.memory.lock().unwrap();
        let get_result = mem.get(&key);
        match get_result {
            Some(value) => Ok(Self::format_simple_string(value.to_owned())),
            None => Err(Box::new(std::fmt::Error)),
        }
    }

    pub fn execute_command(&self, cmd: Command) -> Result<String, Box<dyn std::error::Error>> {
        match cmd {
            Command::Ping => Ok(Self::ping_command()),
            Command::Echo(arg) => Ok(Self::echo_command(arg)),
            Command::Get(key) => self.get_command(key),
            Command::Set(key, value) => self.set_command(key, value),
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

    fn format_simple_string(arg: String) -> String {
        let encoded = format!("+{}\r\n", arg);
        encoded
    }
}
