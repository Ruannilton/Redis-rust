use crate::{
    redis::{
        redis_app::RedisApp,
        redis_error::RedisError,
        types::{entry_value::EntryValue, value_container::ValueContainer},
    },
    resp::resp_serializer::to_resp_string,
    utils,
};

use super::command_trait::Command;

pub struct SetCommand {
    key: String,
    value: ValueContainer,
    expiration: Option<u128>,
}

impl SetCommand {
    pub fn new(key: String, value: ValueContainer, expiration: Option<u128>) -> Self {
        Self {
            key,
            value,
            expiration,
        }
    }
}

impl Command for SetCommand {
    async fn execute(self, app: &RedisApp) -> Result<String, RedisError> {
        let mut mem = app.memory.lock().await;

        let expires: Option<u128> = match self.expiration {
            Some(ex) => Some(utils::get_current_time_ms() + ex),
            None => None,
        };

        let entry = EntryValue {
            value: self.value,
            expires_at: expires,
        };

        _ = mem.insert(self.key, entry);
        Ok(to_resp_string("OK".to_owned()))
    }
}
