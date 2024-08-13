use crate::{
    redis::{redis_app::RedisApp, redis_error::RedisError},
    resp::resp_serializer::to_resp_bulk,
};

use super::command_trait::Command;

pub struct PingCommand {}

impl PingCommand {
    pub fn new() -> Self {
        Self {}
    }
}

impl Command for PingCommand {
    async fn execute(self, _: &RedisApp) -> Result<String, RedisError> {
        Ok(to_resp_bulk("PONG".to_owned()))
    }
}
