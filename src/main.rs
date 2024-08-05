mod input_parser;
mod output_parser;
mod rdb_file;
mod redis_app;
mod redis_parser;
mod redis_types;
mod resp_desserializer;
mod resp_desserializer_error;
mod resp_invalid_command_error;
mod resp_type;
mod web_api;
use redis_app::RedisApp;
use resp_desserializer::RespDesserializer;
use std::{env, sync::Arc};
use web_api::RedisServer;
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let _desses = RespDesserializer {};
    let args = env::args().skip(1);

    let redis_app = Arc::new(RedisApp::new(args));

    let server = RedisServer::new("127.0.0.1:6379", redis_app).await?;

    server.run().await?;

    Ok(())
}
