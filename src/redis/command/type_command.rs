use crate::{
    redis::{redis_error::RedisError, types::value_container::ValueContainer},
    resp::resp_serializer::to_resp_string,
};

use super::command_trait::Command;

pub struct TypeCommand {
    key: String,
}

impl TypeCommand {
    pub fn new(key: String) -> Self {
        Self { key }
    }
}

impl Command for TypeCommand {
    async fn execute(&self, app: &crate::redis::redis_app::RedisApp) -> Result<String, RedisError> {
        let mem = app.memory.lock().await;

        if let Some(value) = mem.get(&self.key).and_then(|entry| entry.get_value()) {
            match value {
                ValueContainer::Stream(..) => Ok(to_resp_string("stream".to_owned())),
                ValueContainer::String(_) => Ok(to_resp_string("string".to_owned())),
                ValueContainer::Array(..) => Ok(to_resp_string("list".to_owned())),
                ValueContainer::Integer(_) => Ok(to_resp_string("integer".to_owned())),
            }
        } else {
            Ok(to_resp_string("none".to_owned()))
        }
    }
}
