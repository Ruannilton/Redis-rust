use std::sync::Arc;

use crate::{
    resp::resp_serializer,
    resp_desserializer::RespTk,
    server::{actions::full_resync::full_resync, redis_app::RedisApp},
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

    let mut deferred = app.deferred_actions.lock().await;
    deferred.insert(context.connection_id, full_resync);
    serialized
}
