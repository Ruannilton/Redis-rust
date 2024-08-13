use crate::{
    redis::redis_error::RedisError,
    resp::resp_serializer::{null_resp_string, to_resp_array},
};

use super::command_trait::Command;

pub struct ConfiGet {
    key: String,
}

impl ConfiGet {
    pub fn new(key: String) -> Self {
        Self { key }
    }
}

impl Command for ConfiGet {
    async fn execute(self, app: &crate::redis::redis_app::RedisApp) -> Result<String, RedisError> {
        let configs = app
            .configurations
            .lock()
            .map_err(|_| RedisError::LockError)?;

        if let Some(value) = configs.get(&self.key) {
            let value_arr = vec![self.key, value.to_owned()];
            Ok(to_resp_array(value_arr))
        } else {
            Ok(null_resp_string())
        }
    }
}
