use crate::{
    redis::{redis_app::RedisApp, redis_error::RedisError},
    resp::resp_serializer::{null_resp_string, to_resp_string},
};

use super::command_trait::Command;

pub struct GetCommand {
    key: String,
}

impl GetCommand {
    pub fn new(key: String) -> Self {
        Self { key }
    }
}

impl Command for GetCommand {
    async fn execute(&self, app: &RedisApp) -> Result<String, RedisError> {
        let map = app.memory.lock().await;

        if let Some(value) = map.get(&self.key).and_then(|f| f.get_value()) {
            let value_str: String = value.into();
            Ok(to_resp_string(value_str))
        } else {
            Ok(null_resp_string())
        }
    }
}
