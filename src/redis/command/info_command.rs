use std::ops::Deref;

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
        let app_type_mutex = app.instance_type.lock().await;
        let app_type = app_type_mutex.deref();

        let instance_type = match app_type {
            crate::redis::types::instance_type::InstanceType::Master => "master",
            crate::redis::types::instance_type::InstanceType::Slave => "slave",
        };

        let resp = format!("# Replication\nrole:{}", instance_type);

        let resp = to_resp_bulk(resp);

        Ok(resp)
    }
}

impl InfoCommand {
    pub fn new(key: String) -> Self {
        InfoCommand { key }
    }
}
