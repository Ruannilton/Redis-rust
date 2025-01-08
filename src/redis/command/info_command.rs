use crate::{
    redis::{redis_app::RedisApp, redis_error::RedisError},
    resp::resp_serializer::to_resp_bulk,
};

use super::command_trait::Command;

pub struct InfoCommand {
    key: String,
}

impl Command for InfoCommand {
    async fn execute(self, _: &RedisApp) -> Result<String, RedisError> {
        let resp = String::from("# Replication\nrole:master");

        let resp = to_resp_bulk(resp);

        Ok(resp)
    }
}

impl InfoCommand {
    pub fn new(key: String) -> Self {
        InfoCommand { key }
    }
}
