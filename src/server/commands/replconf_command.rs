use std::sync::Arc;

use crate::{
    resp::resp_serializer,
    resp_desserializer::RespTk,
    server::redis_app::RedisApp,
    types::{
        connection_context::ConnectionContext, execution_response::ExecResponse,
        redis_replica::RedisReplica,
    },
};

use super::command_utils;

pub async fn execute_replconf(
    app: Arc<RedisApp>,
    token: &RespTk,
    context: ConnectionContext,
) -> ExecResponse {
    let mut args = token.get_command_args();
    let cmd = command_utils::get_next_arg_string(&mut args);
    let val = command_utils::get_next_arg_string(&mut args);

    if let (Some(cmd), Some(val)) = (cmd, val) {
        match cmd.as_str() {
            "listening-port" => {
                let port = val;
                let replica = RedisReplica::new(context.client_address, port);
                app.add_replica(replica).await;
            }
            _ => {}
        }
    }
    let response = resp_serializer::to_resp_string("OK".into());
    response.into()
}
