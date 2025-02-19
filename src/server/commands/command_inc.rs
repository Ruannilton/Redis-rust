use std::sync::Arc;

use crate::{
    resp::resp_serializer,
    resp_desserializer::RespTk,
    server::redis_app::RedisApp,
    types::{
        entry_value::EntryValue, execution_response::ExecResponse, value_container::ValueContainer,
    },
};

use super::command_utils;

pub async fn execute_inc(app: Arc<RedisApp>, token: &RespTk) -> ExecResponse {
    let mut args = token.get_command_args();
    let mut mem = app.memory.lock().await;
    let key_op = command_utils::get_next_arg_string(&mut args);

    if let Some(entry) = key_op.clone().and_then(|key| mem.get_mut(&key)) {
        match entry.value.to_owned() {
            ValueContainer::String(str) => {
                if let Ok(i) = i64::from_str_radix(str.as_str(), 10) {
                    let nv = i + 1;
                    entry.value = ValueContainer::String(nv.to_string());
                    app.buffer_command(token).await;
                    return resp_serializer::to_resp_integer(nv).into();
                } else {
                    return resp_serializer::to_err_string(
                        "ERR value is not an integer or out of range".into(),
                    )
                    .into();
                }
            }
            ValueContainer::Integer(i) => {
                entry.value = ValueContainer::Integer(i + 1);
                app.buffer_command(token).await;
                return resp_serializer::to_resp_integer(i + 1).into();
            }
            _ => {
                return resp_serializer::to_err_string(
                    "ERR value is not an integer or out of range".into(),
                )
                .into();
            }
        }
    } else if let Some(key) = key_op.clone() {
        let entry = EntryValue {
            expires_at: None,
            value: ValueContainer::Integer(1),
        };
        mem.insert(key, entry);
        app.buffer_command(token).await;
        return resp_serializer::to_resp_integer(1).into();
    }
    resp_serializer::to_err_string("ERR value is not an integer or out of range".into()).into()
}
