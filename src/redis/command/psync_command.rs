use crate::{
    redis::{redis_app::RedisApp, redis_error::RedisError},
    resp::resp_serializer,
};

use super::command_trait::Command;

pub struct PsyncCommand {
    replication_id: String,
    offset: i64,
}

impl PsyncCommand {
    pub fn new(replid: String, offset: i64) -> Self {
        Self {
            replication_id: replid,
            offset: offset,
        }
    }
}

impl Command for PsyncCommand {
    async fn execute(&self, app: &RedisApp) -> Result<String, RedisError> {
        let def = String::new();
        let replid = app.settings.master_replid.as_ref().unwrap_or(&def);
        let response: String = format!("FULLRESYNC {} {}\r\n", replid, 0);
        let serialized = resp_serializer::to_resp_string(response);
        Ok(serialized)
    }
}
