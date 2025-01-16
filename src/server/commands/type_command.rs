use std::sync::Arc;

use crate::{
    resp::resp_serializer,
    resp_desserializer::RespTk,
    server::redis_app::RedisApp,
    types::{execution_response::ExecResponse, value_container::ValueContainer},
};

pub async fn execute_type(app: Arc<RedisApp>, token: &RespTk) -> ExecResponse {
    let mem = app.memory.lock().await;

    if let Some(value) = token
        .get_command_args()
        .next()
        .and_then(|tk| tk.get_content_string())
        .and_then(|key| mem.get(&key))
        .and_then(|entry| entry.get_value())
    {
        let resp = match value {
            ValueContainer::Stream(..) => resp_serializer::to_resp_string("stream".to_owned()),
            ValueContainer::String(_) => resp_serializer::to_resp_string("string".to_owned()),
            ValueContainer::Array(..) => resp_serializer::to_resp_string("list".to_owned()),
            ValueContainer::Integer(_) => resp_serializer::to_resp_string("integer".to_owned()),
            ValueContainer::Boolean(_) => resp_serializer::to_resp_string("boolean".to_owned()),
            ValueContainer::Null => resp_serializer::to_resp_string("none".to_owned()),
        };
        resp.into()
    } else {
        resp_serializer::to_resp_string("none".to_owned()).into()
    }
}
