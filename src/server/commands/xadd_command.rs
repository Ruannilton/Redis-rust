use std::sync::Arc;

use crate::{
    resp::resp_serializer,
    resp_desserializer::RespTk,
    server::redis_app::RedisApp,
    types::{
        entry_value::EntryValue, execution_response::ExecResponse, stream_entry::StreamEntry,
        stream_key::StreamKey, value_container::ValueContainer,
    },
};

pub async fn execute_xadd(app: Arc<RedisApp>, token: &RespTk) -> ExecResponse {
    let mut args = token.get_command_args();
    if let (Some(stream_id), Some(entry_id)) = (
        args.next().and_then(|t| t.get_content_string()),
        args.next().and_then(|t| t.get_content_string()),
    ) {
        let mut fields = Vec::new();

        while let (Some(key), Some(value)) = (
            args.next().and_then(|t| t.get_content_string()),
            args.next().and_then(|t| t.get_content_string()),
        ) {
            fields.push((key, value));
        }

        return execute(token, app, stream_id, entry_id, fields).await;
    }
    resp_serializer::null_resp_string().into()
}

async fn execute(
    token: &RespTk,
    app: Arc<RedisApp>,
    stream_id: String,
    entry_id: String,
    fields: Vec<(String, String)>,
) -> ExecResponse {
    if entry_id == "0-0" {
        return resp_serializer::to_err_string(
            "ERR The ID specified in XADD must be greater than 0-0".to_owned(),
        )
        .into();
    }

    let mut mem = app.memory.lock().await;
    let last_key = app.get_last_stream_key(&stream_id, &mem);
    let stream_key_result = StreamKey::from_string(&entry_id.to_owned(), &last_key, None);

    if stream_key_result.is_err() {
        return resp_serializer::to_err_string("INVALID_COMMAND".into()).into();
    }

    let stream_key = stream_key_result.unwrap();

    if let Some(last) = last_key {
        if stream_key <= last {
            return resp_serializer::to_err_string(String::from(
                "ERR The ID specified in XADD is equal or smaller than the target stream top item",
            ))
            .into();
        }
    }

    let new_entry = StreamEntry {
        id: stream_key.clone(),
        fields: fields.to_owned(),
    };

    if let Some(entry) = mem.get_mut(&stream_id) {
        if let ValueContainer::Stream(ref mut stream) = entry.value {
            stream.push(new_entry);
            app.broadcast_command(token).await;
            return resp_serializer::to_resp_bulk(stream_key.into()).into();
        }
    }

    mem.insert(
        stream_id.to_owned(),
        EntryValue {
            expires_at: None,
            value: ValueContainer::Stream(vec![new_entry]),
        },
    );
    app.broadcast_command(token).await;
    resp_serializer::to_resp_bulk(stream_key.into()).into()
}
