use hex;
use std::sync::Arc;

use crate::{server::redis_app::RedisApp, utils};

pub fn full_resync(_app: Arc<RedisApp>) -> String {
    let hex = "524544495330303131fa0972656469732d76657205372e322e30fa0a72656469732d62697473c040fa056374696d65c26d08bc65fa08757365642d6d656dc2b0c41000fa08616f662d62617365c000fff06e3bfec0ff5aa2";
    let file = hex::decode(hex).unwrap();

    let mut response: String = format!("${}\r\n", file.len());
    response.push_str(&String::from_utf8_lossy(&file));
    response
}
