mod redis;
mod redis_server;
mod resp;
mod utils;
use redis::redis_app::RedisApp;
use redis_server::RedisServer;
use std::{env, sync::Arc};
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = env::args().skip(1);

    let redis_app = Arc::new(RedisApp::new(args));

    let server = RedisServer::new("127.0.0.1", redis_app).await?;

    server.run().await?;

    Ok(())
}
