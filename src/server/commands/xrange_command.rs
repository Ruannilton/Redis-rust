use std::sync::Arc;

use crate::{
    resp::resp_serializer,
    resp_desserializer::RespTk,
    server::redis_app::RedisApp,
    types::{redis_error::RedisError, stream_key::StreamKey, value_container::ValueContainer},
};

pub async fn execute_xrange(app: Arc<RedisApp>, token: &RespTk) -> String {
    let mut args = token.get_command_args();
    if let (Some(stream_id), Some(start), Some(end)) = (
        args.next().and_then(|t| t.get_content_string()),
        args.next().and_then(|t| t.get_content_string()),
        args.next().and_then(|t| t.get_content_string()),
    ) {
        let mem = app.memory.lock().await;
        let start_id = StreamKey::from_string(&start, &None, Some(0))
            .map_err(|_| RedisError::InvalidStreamEntryId(start))
            .unwrap();
        let end_id = StreamKey::from_string(&end, &None, Some(u64::MAX))
            .map_err(|_| RedisError::InvalidStreamEntryId(end))
            .unwrap();

        if end_id < start_id {
            return resp_serializer::to_err_string(String::from("ERR Invalid range"));
        }

        if let Some(entry_value) = mem.get(&stream_id) {
            if let ValueContainer::Stream(stream) = &entry_value.value {
                let idx_start = match stream.binary_search_by(|val| val.id.cmp(&start_id)) {
                    Ok(idx) => idx,
                    Err(idx) => idx,
                };

                let idx_end = match stream.binary_search_by(|val| val.id.cmp(&end_id)) {
                    Ok(idx) => idx + 1,
                    Err(idx) => idx,
                };

                let slice = &stream[idx_start..idx_end];
                let serialized = resp_serializer::slc_objects_to_resp(slice);
                return serialized;
            }
        }

        return resp_serializer::to_err_string(String::from("ERR The ID specified not exists"));
    }
    resp_serializer::null_resp_string()
}
