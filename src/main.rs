pub mod rdb;
mod resp;
mod server;
pub mod types;
mod utils;

use resp::{resp_desserializer, resp_serializer};
use server::redis_app::RedisApp;
use std::{env, sync::Arc};
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::TcpStream,
};
use types::{connection_context::ConnectionContext, instance_type::InstanceType};
#[tokio::main]

async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = env::args().skip(1);

    let redis_app = Arc::new(RedisApp::new(args));
    let listener_address = format!("{}:{}", "127.0.0.1", redis_app.settings.port);
    let tcp_listener = tokio::net::TcpListener::bind(listener_address).await?;
    let mut connection_counter: u64 = 0;

    match redis_app.get_istance_type() {
        InstanceType::Slave => do_handshake(redis_app.clone()).await?,
        _ => {}
    }

    loop {
        if let Ok((connection_stream, _)) = tcp_listener.accept().await {
            println!("Connection received");
            connection_counter += 1;
            let app = redis_app.clone();
            tokio::spawn(async move {
                if let Err(e) = handle_request(connection_stream, app, connection_counter).await {
                    eprintln!("Error handling request: {:?}", e);
                }
            });
        }
    }
}

async fn handle_request(
    mut stream: TcpStream,
    app: Arc<RedisApp>,
    connection_id: u64,
) -> Result<(), Box<dyn std::error::Error>> {
    loop {
        let mut stream_buffer = vec![0; 1024];
        let read_result = stream.read(&mut stream_buffer).await?;

        if read_result == 0 {
            return Ok(());
        }

        println!(
            "Received: {:?}",
            String::from_utf8_lossy(&stream_buffer[..read_result])
        );

        if let Some(token) = resp_desserializer::parse_resp_buffer(&stream_buffer[..read_result]) {
            let conn_addr = stream.peer_addr().unwrap().ip().to_string();
            let context = ConnectionContext::new(connection_id, conn_addr);
            let response =
                server::command_executor::execute_command(app.clone(), &token, context).await;
            let response_buffer = response.into_bytes();
            stream.write_all(response_buffer.as_slice()).await?;
            stream.flush().await?;

            let deferred = app.deferred_actions.lock().await;

            if let Some(actions) = deferred.get(&connection_id) {
                for action in actions {
                    let response = action(app.clone());
                    let response_buffer = response.into_bytes();
                    stream.write_all(response_buffer.as_slice()).await?;
                    stream.flush().await?;
                }
            }
        }
    }
}

async fn do_handshake(app: Arc<RedisApp>) -> Result<(), Box<dyn std::error::Error>> {
    let master_address = app.get_master_conn().unwrap();
    let mut stream = TcpStream::connect(master_address).await?;
    let mut buffer = vec![0; 1024];

    let ping_payload = resp_serializer::to_resp_array(vec!["PING".into()]).into_bytes();
    stream.write_all(&ping_payload).await?;
    stream.read(&mut buffer).await?;

    let replconf_payload = resp_serializer::to_resp_array(vec![
        "REPLCONF".into(),
        "listening-port".into(),
        app.settings.port.to_string(),
    ])
    .into_bytes();
    stream.write_all(&replconf_payload).await?;
    stream.read(&mut buffer).await?;

    let replconf2_payload =
        resp_serializer::to_resp_array(vec!["REPLCONF".into(), "capa".into(), "psync2".into()])
            .into_bytes();
    stream.write_all(&replconf2_payload).await?;
    stream.read(&mut buffer).await?;

    let psync_payload =
        resp_serializer::to_resp_array(vec!["PSYNC".into(), "?".into(), "-1".into()]).into_bytes();

    stream.write_all(&psync_payload).await?;
    stream.read(&mut buffer).await?;
    Ok(())
}
