use std::sync::Arc;

use crate::{
    resp::resp_serializer, resp_desserializer::RespTk, server::redis_app::RedisApp,
    types::connection_context::ConnectionContext,
};

pub async fn execute_psync(
    app: Arc<RedisApp>,
    _token: &RespTk,
    context: ConnectionContext,
) -> String {
    let def = String::new();
    let replid = app.settings.master_replid.as_ref().unwrap_or(&def);
    let response: String = format!("FULLRESYNC {} {}", replid, 0);
    let serialized = resp_serializer::to_resp_string(response);
    add_resync(app, context).await;
    serialized
}

const EMPTY_RDB_HEX :&str = "524544495330303131fa0972656469732d76657205372e322e30fa0a72656469732d62697473c040fa056374696d65c26d08bc65fa08757365642d6d656dc2b0c41000fa08616f662d62617365c000fff06e3bfec0ff5aa2";

async fn add_resync(app: Arc<RedisApp>, context: ConnectionContext) {
    let mut deferred = app.deferred_actions.lock().await;
    deferred.insert(context.connection_id, init_resync);
    deferred.insert(context.connection_id, end_resync);
}

fn init_resync(_app: Arc<RedisApp>) -> String {
    let file = hex::decode(EMPTY_RDB_HEX).unwrap();

    let response: String = format!("${}\r\n", file.len());
    return response;
}
fn end_resync(_app: Arc<RedisApp>) -> String {
    let file = hex::decode(EMPTY_RDB_HEX).unwrap();
    String::from_utf8_lossy(&file).to_string()
}
