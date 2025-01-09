use crate::{redis::redis_error::RedisError, resp::resp_serializer::to_resp_array};

use super::command_trait::Command;

pub struct KeysCommand {}

impl KeysCommand {
    pub fn new(_key: String) -> Self {
        Self {}
    }
}

impl Command for KeysCommand {
    async fn execute(&self, app: &crate::redis::redis_app::RedisApp) -> Result<String, RedisError> {
        let mem = app.memory.lock().await;

        let keys: Vec<&String> = mem.keys().collect();
        let keys_owned: Vec<String> = keys.iter().map(|s| s.to_owned().to_owned()).collect();
        Ok(to_resp_array(keys_owned))
    }
}
