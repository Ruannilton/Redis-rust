use crate::{
    redis::{redis_app::RedisApp, redis_error::RedisError},
    resp::resp_serializer::to_resp_string,
};

use super::command_trait::Command;

pub struct ExecCommand {}

impl ExecCommand {
    pub fn new() -> Self {
        Self {}
    }
}

impl Command for ExecCommand {
    async fn execute(self, _: &RedisApp) -> Result<String, RedisError> {
        Ok(to_resp_string("OK".to_owned()))
    }
}
