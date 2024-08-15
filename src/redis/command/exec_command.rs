use crate::{
    redis::{redis_app::RedisApp, redis_error::RedisError},
    resp::resp_serializer::{to_err_string, to_resp_string},
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
        Ok(to_err_string("ERR EXEC without MULTI".to_owned()))
    }
}
