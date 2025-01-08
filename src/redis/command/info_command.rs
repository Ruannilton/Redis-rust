use crate::{
    redis::{redis_app::RedisApp, redis_error::RedisError},
    resp::resp_serializer::to_resp_bulk,
};

use super::command_trait::Command;

pub struct InfoCommand {
    key: String,
}

impl Command for InfoCommand {
    async fn execute(self, app: &RedisApp) -> Result<String, RedisError> {
        let mut response_str = String::new();

        response_str.push_str("# Replication\n");

        let instance_type = match app.settings.instance_type {
            crate::redis::types::instance_type::InstanceType::Master => "role:master",
            crate::redis::types::instance_type::InstanceType::Slave => "role:slave",
        };

        response_str.push_str(instance_type);

        if let Some(master_replid) = &app.settings.master_replid {
            let replid = format!("\nmaster_replid:{}", master_replid);
            response_str.push_str(replid.as_str());
        }

        let master_repl_offset =
            format!("\nmaster_repl_offset:{}", app.settings.master_repl_offset);
        response_str.push_str(master_repl_offset.as_str());

        let resp = to_resp_bulk(response_str);

        Ok(resp)
    }
}

impl InfoCommand {
    pub fn new(key: String) -> Self {
        InfoCommand { key }
    }
}
