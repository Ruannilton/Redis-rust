use super::command_trait::Command;
use crate::{
    redis::{redis_app::RedisApp, redis_error::RedisError},
    resp::resp_serializer::to_resp_bulk,
};

pub struct EchoCommand {
    echo_val: String,
}

impl EchoCommand {
    pub fn new(value: String) -> Self {
        Self { echo_val: value }
    }
}

impl Command for EchoCommand {
    async fn execute(&self, _: &RedisApp) -> Result<String, RedisError> {
        Ok(to_resp_bulk(self.echo_val.to_owned()))
    }
}
