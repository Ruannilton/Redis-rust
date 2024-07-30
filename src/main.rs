mod parser;
mod redis_cli;
mod web_api;
mod rdb_file;

use std::{env, sync::Arc};

use redis_cli::RedisApp;
use web_api::RedisServer;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = env::args().skip(1);

    let redis_app = Arc::new(RedisApp::new(args));

    let server = RedisServer::new("127.0.0.1:6379", redis_app).await?;

    server.run().await?;

    Ok(())
}
