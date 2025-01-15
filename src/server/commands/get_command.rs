use std::sync::Arc;

use crate::{resp::resp_serializer, resp_desserializer::RespTk, server::redis_app::RedisApp};

pub async fn execute_get(app: Arc<RedisApp>, tk: &RespTk) -> String {
    if let Some(key) = tk
        .get_command_args()
        .next()
        .and_then(|t| t.get_content_string())
    {
        if let Some(entry) = app.get_entry(&key).await {
            let value: String = entry.into();
            return resp_serializer::to_resp_string(value);
        }
    }
    resp_serializer::null_resp_string()
}
