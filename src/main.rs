mod redis;
mod resp;
mod utils;
use redis::{
    redis_app::RedisApp,
    redis_parser,
    types::{command_token::CommandToken, instance_type::InstanceType},
};
use resp::{resp_desserializer, resp_serializer};
use std::{env, sync::Arc};
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::TcpStream,
};
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
            connection_counter += 1;
            let app = redis_app.clone();
            tokio::spawn(async move {
                handle_request(connection_stream, app, connection_counter).await
            });
        }
    }
}

async fn handle_request(mut stream: TcpStream, app: Arc<RedisApp>, connection_id: u64) {
    let mut stream_buffer = [0; 1024];
    let read_result = stream.read(&mut stream_buffer).await.unwrap();

    if read_result == 0 {
        return;
    }

    let commands = extract_commands(&stream_buffer[..read_result]).unwrap();

    for command in commands {
        execute_command(command, app.clone(), connection_id, &mut stream).await;
    }
}

async fn execute_command(
    cmd: CommandToken,
    app: Arc<RedisApp>,
    connection_id: u64,
    stream: &mut TcpStream,
) {
    {
        let mut transactions = app.transactions.lock().await;

        if let Some(tsx) = transactions.get_mut(&connection_id) {
            match cmd {
                CommandToken::Exec | CommandToken::Discard => {}
                _ => {
                    tsx.push_back(cmd);
                    let response = resp_serializer::to_resp_string("QUEUED".into()).into_bytes();
                    let _ = stream.write(&response).await;
                    return;
                }
            }
        }
    }

    let exec_result = cmd.execute(connection_id, app).await;
    let response_buffer = exec_result.into_bytes();
    let _ = stream.write(response_buffer.as_slice()).await;
}

fn extract_commands(buffer: &[u8]) -> Result<Vec<CommandToken>, Box<dyn std::error::Error>> {
    let mut tokens = resp_desserializer::desserialize(buffer)?;
    let commands = redis_parser::parse_token_into_command(&mut tokens)?;
    Ok(commands)
}

async fn do_handshake(app: Arc<RedisApp>) -> Result<(), Box<dyn std::error::Error>> {
    let master_address = app.get_master_conn().unwrap();
    let mut stream = TcpStream::connect(master_address).await?;
    let mut buffer = [0; 1024];

    let ping_payload = resp_serializer::to_resp_array(vec!["PING".into()]).into_bytes();
    stream.write_all(&ping_payload).await?;
    _ = stream.read(&mut buffer).await?;

    let replconf_payload = resp_serializer::to_resp_array(vec![
        "REPLCONF".into(),
        "listening-port".into(),
        app.settings.port.to_string(),
    ])
    .into_bytes();
    stream.write_all(&replconf_payload).await?;
    _ = stream.read(&mut buffer).await?;

    let replconf2_payload =
        resp_serializer::to_resp_array(vec!["REPLCONF".into(), "capa".into(), "psync2".into()])
            .into_bytes();
    stream.write_all(&replconf2_payload).await?;
    _ = stream.read(&mut buffer).await?;

    let psync_payload =
        resp_serializer::to_resp_array(vec!["PSYNC".into(), "?".into(), "-1".into()]).into_bytes();

    stream.write_all(&psync_payload).await?;
    _ = stream.read(&mut buffer).await?;
    Ok(())
}
