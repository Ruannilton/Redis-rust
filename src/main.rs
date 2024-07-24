mod parser;
mod redis_cli;
mod web_api;

use web_api::RedisServer;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let server = RedisServer::new("127.0.0.1:6379").await?;

    server.run().await?;

    Ok(())
}
