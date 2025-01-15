use std::sync::Arc;

use crate::{resp::resp_serializer, resp_desserializer::RespTk, server::redis_app::RedisApp};

pub async fn execute_keys(app: Arc<RedisApp>, _token: &RespTk) -> String {
    let mem = app.memory.lock().await;

    let keys: Vec<&String> = mem.keys().collect();
    let keys_owned: Vec<String> = keys.iter().map(|s| s.to_owned().to_owned()).collect();
    resp_serializer::to_resp_array(keys_owned)
}
