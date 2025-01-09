use std::collections::VecDeque;

use crate::{
    redis::{redis_app::RedisApp, redis_error::RedisError, types::transactions::ClientId},
    resp::resp_serializer::to_resp_string,
};

use super::command_trait::Command;

pub struct MultiCommand {
    client_id: ClientId,
}

impl MultiCommand {
    pub fn new(client_id: ClientId) -> Self {
        Self { client_id }
    }
}

impl Command for MultiCommand {
    async fn execute(&self, app: &RedisApp) -> Result<String, RedisError> {
        let mut transactions = app.transactions.lock().await;
        if !transactions.contains_key(&self.client_id) {
            let transaction = VecDeque::new();
            transactions.insert(self.client_id, transaction);
        }
        Ok(to_resp_string("OK".to_owned()))
    }
}
