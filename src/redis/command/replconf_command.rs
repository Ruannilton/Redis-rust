use crate::resp::resp_serializer;

use super::command_trait::Command;

pub struct ReplConfCommand {
    arguments: Vec<String>,
}

impl Command for ReplConfCommand {
    async fn execute(
        &self,
        _: &crate::redis::redis_app::RedisApp,
    ) -> Result<String, crate::redis::redis_error::RedisError> {
        let response = resp_serializer::to_resp_string("OK".into());
        Ok(response)
    }
}

impl ReplConfCommand {
    pub fn new(arguments: Vec<String>) -> Self {
        Self { arguments }
    }
}
