use std::sync::Arc;

use crate::{
    resp::resp_serializer, resp_desserializer::RespTk, server::redis_app::RedisApp,
    types::execution_response::ExecResponse,
};

const EMPTY_RDB_HEX :&str = "524544495330303131fa0972656469732d76657205372e322e30fa0a72656469732d62697473c040fa056374696d65c26d08bc65fa08757365642d6d656dc2b0c41000fa08616f662d62617365c000fff06e3bfec0ff5aa2";

pub async fn execute_psync(app: Arc<RedisApp>, _token: &RespTk) -> ExecResponse {
    let def = String::new();
    let replid = app.settings.master_replid.as_ref().unwrap_or(&def);
    let resync: String = format!("FULLRESYNC {} {}", replid, 0);

    let resync_resp = resp_serializer::to_resp_string(resync).into_bytes();
    let file = hex::decode(EMPTY_RDB_HEX).unwrap();
    let file_header = format!("${}\r\n", file.len()).into_bytes();

    let resp = vec![resync_resp, file_header, file];

    resp.into()
}
