use crate::{
    redis::{redis_app::RedisApp, redis_error::RedisError, types::transactions::ClientId},
    resp::resp_serializer::{to_err_string, to_resp_string},
};

use super::command_trait::Command;

pub struct DiscardCommand {
    client_id: ClientId,
}

impl DiscardCommand {
    pub fn new(client_id: ClientId) -> Self {
        Self { client_id }
    }
}

impl Command for DiscardCommand {
    async fn execute(&self, app: &RedisApp) -> Result<String, RedisError> {
        let mut transactions = app.transactions.lock().await;

        match transactions.get(&self.client_id) {
            Some(_) => {
                transactions.remove(&self.client_id);
                return Ok(to_resp_string("OK".to_owned()));
            }
            None => return Ok(to_err_string("ERR DISCARD without MULTI".to_owned())),
        }
    }
}
