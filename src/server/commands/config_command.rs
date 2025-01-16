use std::sync::Arc;

use crate::{
    resp::resp_serializer, resp_desserializer::RespTk, server::redis_app::RedisApp,
    types::execution_response::ExecResponse,
};

pub async fn execute_config(app: Arc<RedisApp>, token: &RespTk) -> ExecResponse {
    if let Some(key) = token
        .get_command_args()
        .next()
        .and_then(|tk| tk.get_content_string())
    {
        let configs = app.settings.to_hashmap();
        if let Some(value) = configs.get(key.as_str()) {
            let value_arr = vec![key.to_owned(), value.to_owned()];
            return resp_serializer::to_resp_array(value_arr).into();
        }
    }

    resp_serializer::null_resp_string().into()
}
