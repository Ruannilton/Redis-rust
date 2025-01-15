use std::sync::Arc;

use crate::{server::redis_app::RedisApp, utils};

pub fn full_resync(_app: Arc<RedisApp>) -> String {
    let base64_file = "UkVESVMwMDEx+glyZWRpcy12ZXIFNy4yLjD6CnJlZGlzLWJpdHPAQPoFY3RpbWXCbQi8ZfoIdXNlZC1tZW3CsMQQAPoIYW9mLWJhc2XAAP/wbjv+wP9aog==";
    let bin_file = utils::base64_to_bin(base64_file).unwrap();
    let mut response: String = format!("${}\r\n", bin_file.len());
    response.push_str(&String::from_utf8_lossy(&bin_file));
    response
}
