use crate::{
    redis::{redis_app::RedisApp, redis_error::RedisError, types::transactions::ClientId},
    resp::resp_serializer::to_err_string,
};

use super::{command_executor::execute_command, command_trait::Command};

pub struct ExecCommand {
    client_id: ClientId,
}

impl ExecCommand {
    pub fn new(client_id: ClientId) -> Self {
        Self { client_id }
    }
}

impl Command for ExecCommand {
    async fn execute(self, app: &RedisApp) -> Result<String, RedisError> {
        let transactions = app.transactions.lock().await;

        match transactions.get(&self.client_id) {
            Some(commands) => {
                let mut responses = Vec::new();
                for cmd in commands {
                    let resp = Box::pin(execute_command(app, self.client_id, cmd.clone())).await?;
                    responses.push(resp);
                }
                let mut response = format!("*{}\r\n", responses.len());
                for r in responses {
                    response.push_str(&r);
                }
                return Ok(response);
            }
            None => return Ok(to_err_string("ERR EXEC without MULTI".to_owned())),
        }
    }
}
