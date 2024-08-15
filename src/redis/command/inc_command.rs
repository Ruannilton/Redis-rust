use crate::{
    redis::{
        redis_app::RedisApp,
        redis_error::RedisError,
        types::{entry_value::EntryValue, value_container::ValueContainer},
    },
    resp::resp_serializer::{to_err_string, to_resp_integer},
};

use super::command_trait::Command;

pub struct IncCommand {
    key: String,
}

impl IncCommand {
    pub fn new(key: String) -> Self {
        Self { key }
    }
}

impl Command for IncCommand {
    async fn execute(self, app: &RedisApp) -> Result<String, RedisError> {
        let mut mem = app.memory.lock().map_err(|_| RedisError::LockError)?;

        let new_val = match mem.get_mut(&self.key) {
            Some(entry) => {
                let new_val = match &entry.value {
                    ValueContainer::String(str) => {
                        i64::from_str_radix(str, 10).map_err(|_| RedisError::ParsingError)? + 1
                    }
                    ValueContainer::Integer(i) => *i + 1,
                    _ => {
                        return Ok(to_err_string(
                            "ERR value is not an integer or out of range".into(),
                        ))
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
                mem.insert(self.key, entry);
                1
            }
        };

        Ok(to_resp_integer(new_val))
    }
}
